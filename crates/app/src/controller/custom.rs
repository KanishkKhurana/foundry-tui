use std::collections::BTreeMap;

use chrono::Local;
use foundry_tui_config::{CustomTemplate, TemplateTool};
use foundry_tui_foundry::{redact_cli_args, ToolEvent, ToolKind, ToolRequest};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{AnvilInstanceStatus, CustomCommandDraft, CustomCommandModal, CustomModalStep, Tab},
    parsing::{
        convert_angle_placeholder_token, extract_placeholders, extract_placeholders_from_token,
        has_placeholder_tokens, infer_param_meta, is_exact_placeholder_token,
        is_secret_placeholder, looks_like_flag, merge_template_and_raw_args,
        normalize_pasted_command, parse_cli_args, parse_template_tool, template_tool_to_tool_kind,
        validate_custom_param_value,
    },
};

use super::AppController;

impl AppController {
    pub(crate) fn open_custom_runner(&mut self) {
        self.model.active_tab = Tab::Custom;
        self.model.custom_modal = Some(CustomCommandModal {
            step: CustomModalStep::TemplatePicker,
            picker_index: 0,
            paste_mode: false,
            paste_input: String::new(),
            editor_index: 0,
            draft: None,
            error: None,
        });
        self.normalize_focus_for_tab();
        self.model.notification =
            Some("command builder opened: pick a preset or paste a forge/cast command".to_string());
    }

    pub(crate) fn select_prev_custom_template(&mut self) {
        let len = self.model.custom_templates.len();
        if len == 0 {
            return;
        }

        if self.model.custom_template_index == 0 {
            self.model.custom_template_index = len - 1;
        } else {
            self.model.custom_template_index -= 1;
        }
    }

    pub(crate) fn select_next_custom_template(&mut self) {
        let len = self.model.custom_templates.len();
        if len == 0 {
            return;
        }

        self.model.custom_template_index = (self.model.custom_template_index + 1) % len;
    }

    pub(crate) fn parse_pasted_template(
        &self,
        input: &str,
    ) -> std::result::Result<CustomTemplate, String> {
        let normalized = normalize_pasted_command(input);
        let mut tokens = parse_cli_args(&normalized)?;
        if tokens.is_empty() {
            return Err(
                "paste a full forge or cast command, for example `cast call ...`".to_string(),
            );
        }

        let binary = tokens.remove(0).to_lowercase();
        let tool = parse_template_tool(&binary)
            .ok_or_else(|| format!("unsupported command `{binary}` (use forge or cast)"))?;
        if !matches!(tool, TemplateTool::Forge | TemplateTool::Cast) {
            return Err(
                "command builder currently accepts only `forge ...` or `cast ...` commands"
                    .to_string(),
            );
        }
        if tokens.is_empty() {
            return Err("missing command args after binary".to_string());
        }

        let mut args_template = Vec::with_capacity(tokens.len());
        for token in tokens {
            args_template.push(convert_angle_placeholder_token(&token));
        }

        let mut params = BTreeMap::new();
        for placeholder in extract_placeholders(&args_template) {
            params
                .entry(placeholder.clone())
                .or_insert_with(|| infer_param_meta(&placeholder));
        }

        Ok(CustomTemplate {
            id: format!(
                "{}-pasted-{}",
                tool.binary(),
                Local::now().timestamp_millis()
            ),
            label: format!("Pasted {} command", tool.binary()),
            tool,
            args_template,
            description: Some("Generated from pasted command".to_string()),
            tags: vec!["pasted".to_string()],
            default_rpc_preset: self.config.foundry.default_rpc_preset.clone(),
            params,
        })
    }

    pub(crate) fn new_custom_draft(&self, template: CustomTemplate) -> CustomCommandDraft {
        let rpc_preset = self.resolve_template_rpc_preset(&template);
        let preset_rpc_url = self.rpc_url_for_preset(&rpc_preset);
        let rpc_url = self.selected_running_anvil_rpc_url().or(preset_rpc_url);
        let mut param_values = template
            .params
            .iter()
            .filter_map(|(key, meta)| meta.default.clone().map(|value| (key.clone(), value)))
            .collect::<BTreeMap<_, _>>();

        if let Some(url) = rpc_url.clone() {
            param_values.entry("rpc_url".to_string()).or_insert(url);
        }

        CustomCommandDraft {
            template,
            rpc_preset,
            raw_args: String::new(),
            param_values,
            merged_args: Vec::new(),
            resolved_args: Vec::new(),
            display_args: Vec::new(),
            display_commandline: String::new(),
            rpc_url,
        }
    }

    fn resolve_template_rpc_preset(&self, template: &CustomTemplate) -> String {
        if let Some(preset) = &template.default_rpc_preset {
            if self.config.rpc_presets.contains_key(preset) {
                return preset.clone();
            }
        }

        if let Some(preset) = &self.config.foundry.default_rpc_preset {
            if self.config.rpc_presets.contains_key(preset) {
                return preset.clone();
            }
        }

        self.config
            .rpc_presets
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "local".to_string())
    }

    pub(crate) fn cycle_rpc_preset(&self, current: &mut String, forward: bool) {
        let presets = self
            .config
            .rpc_presets
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        if presets.is_empty() {
            return;
        }

        let current_index = presets
            .iter()
            .position(|preset| preset == current)
            .unwrap_or(0);
        let next_index = if forward {
            (current_index + 1) % presets.len()
        } else if current_index == 0 {
            presets.len() - 1
        } else {
            current_index - 1
        };

        *current = presets[next_index].clone();
    }

    pub(crate) fn sync_draft_rpc_url_with_preset(
        &self,
        draft: &mut CustomCommandDraft,
        previous_rpc_url: Option<String>,
    ) {
        let has_rpc_placeholder = extract_placeholders(&draft.template.args_template)
            .iter()
            .any(|placeholder| placeholder == "rpc_url");
        if has_rpc_placeholder {
            let should_replace = match draft.param_values.get("rpc_url") {
                Some(value) => {
                    let value = value.trim();
                    value.is_empty()
                        || previous_rpc_url
                            .as_ref()
                            .is_some_and(|previous| value == previous.trim())
                }
                None => true,
            };

            if should_replace {
                if let Some(next_rpc_url) = self.rpc_url_for_preset(&draft.rpc_preset) {
                    draft
                        .param_values
                        .insert("rpc_url".to_string(), next_rpc_url);
                }
            }
        }

        draft.rpc_url = draft
            .param_values
            .get("rpc_url")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| self.rpc_url_for_preset(&draft.rpc_preset));
    }

    pub(crate) fn prepare_custom_preview(
        &self,
        modal: &mut CustomCommandModal,
    ) -> std::result::Result<(), String> {
        let Some(draft) = modal.draft.as_mut() else {
            return Err("missing draft state".to_string());
        };

        let raw_tokens = if draft.raw_args.trim().is_empty() {
            Vec::new()
        } else {
            parse_cli_args(&draft.raw_args)?
        };

        draft.merged_args = merge_template_and_raw_args(&draft.template.args_template, &raw_tokens);
        self.build_custom_preview(draft)?;
        modal.step = CustomModalStep::Preview;
        Ok(())
    }

    fn build_custom_preview(
        &self,
        draft: &mut CustomCommandDraft,
    ) -> std::result::Result<(), String> {
        let preset_rpc_url = self.rpc_url_for_preset(&draft.rpc_preset);
        let fallback_rpc_url = self.selected_running_anvil_rpc_url().or(preset_rpc_url);
        if let Some(url) = fallback_rpc_url.clone() {
            draft
                .param_values
                .entry("rpc_url".to_string())
                .or_insert(url);
        }

        let mut resolved_args = Vec::with_capacity(draft.merged_args.len());
        let mut display_args = Vec::with_capacity(draft.merged_args.len());

        for (index, token) in draft.merged_args.iter().enumerate() {
            let placeholders = extract_placeholders_from_token(token);
            if placeholders.is_empty() {
                resolved_args.push(token.clone());
                display_args.push(token.clone());
                continue;
            }

            let mut resolved = token.clone();
            let mut display = token.clone();
            let mut skip_token = false;

            for placeholder in placeholders {
                let meta = draft.template.params.get(&placeholder);
                let optional = meta
                    .map(|meta| meta.optional)
                    .unwrap_or_else(|| infer_param_meta(&placeholder).optional);
                let resolved_value = match draft.param_values.get(&placeholder) {
                    Some(value) if !value.trim().is_empty() => Some(value.clone()),
                    Some(_) => None,
                    None => meta
                        .and_then(|meta| meta.default.clone())
                        .filter(|value| !value.trim().is_empty()),
                };

                let Some(value) = resolved_value else {
                    if optional {
                        skip_token = true;
                        break;
                    }
                    return Err(format!("missing value for `{{{{{placeholder}}}}}`"));
                };

                if let Some(error) = validate_custom_param_value(&placeholder, &value, meta) {
                    return Err(error);
                }

                let marker = format!("{{{{{placeholder}}}}}");
                resolved = resolved.replace(&marker, &value);

                let display_value = if is_secret_placeholder(&draft.template, &placeholder) {
                    "******".to_string()
                } else {
                    value.clone()
                };
                display = display.replace(&marker, &display_value);
            }

            if skip_token {
                let previous_is_flag = index > 0
                    && looks_like_flag(&draft.merged_args[index - 1])
                    && is_exact_placeholder_token(token);
                if previous_is_flag {
                    resolved_args.pop();
                    display_args.pop();
                }
                continue;
            }

            if has_placeholder_tokens(&resolved) {
                return Err(format!("unresolved placeholder in `{resolved}`"));
            }

            resolved_args.push(resolved);
            display_args.push(display);
        }

        if resolved_args.is_empty() {
            return Err(format!(
                "{} command cannot be empty",
                draft.template.tool.binary()
            ));
        }

        validate_broadcast_private_key_usage(draft, &resolved_args)?;

        let effective_rpc_url = extract_rpc_url_from_args(&resolved_args)
            .or_else(|| {
                draft
                    .param_values
                    .get("rpc_url")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .or(fallback_rpc_url)
            .or_else(|| self.default_rpc_target());

        draft.resolved_args = resolved_args;
        let redacted_display_args = redact_cli_args(&display_args);
        draft.display_args = redacted_display_args.clone();
        draft.display_commandline = format!(
            "{} {}",
            draft.template.tool.binary(),
            redacted_display_args.join(" ")
        )
        .trim()
        .to_string();
        draft.rpc_url = effective_rpc_url;

        Ok(())
    }

    pub(crate) fn run_custom_draft(
        &mut self,
        draft: &CustomCommandDraft,
        tool_events: &UnboundedSender<ToolEvent>,
    ) -> std::result::Result<u64, String> {
        if !matches!(
            draft.template.tool,
            TemplateTool::Forge | TemplateTool::Cast
        ) {
            return Err("command builder supports only forge and cast presets".to_string());
        }

        let tool = template_tool_to_tool_kind(draft.template.tool);
        let mut request = ToolRequest::new(
            tool,
            draft.resolved_args.clone(),
            self.model.project_root.clone(),
        );
        request.display_commandline = Some(draft.display_commandline.clone());

        if matches!(tool, ToolKind::Forge) {
            request.profile = Some(self.config.foundry.profile.clone());
        }

        if matches!(tool, ToolKind::Forge | ToolKind::Cast) {
            request.rpc_target = draft.rpc_url.clone().or_else(|| self.default_rpc_target());
        }

        let label = format!("Command Builder: {}", draft.template.label);
        let job_id = self.start_tool_request_job(&label, request, tool_events)?;
        self.model.notification = Some(format!("queued builder command as job #{job_id}"));
        Ok(job_id)
    }

    pub(crate) fn rpc_url_for_preset(&self, preset: &str) -> Option<String> {
        self.config.rpc_presets.get(preset).cloned()
    }

    fn selected_running_anvil_rpc_url(&self) -> Option<String> {
        self.model
            .anvil_instances
            .get(self.model.selected_anvil_index)
            .and_then(|instance| {
                if matches!(
                    instance.status,
                    AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
                ) {
                    Some(format!("http://127.0.0.1:{}", instance.port))
                } else {
                    None
                }
            })
    }
}

fn validate_broadcast_private_key_usage(
    draft: &CustomCommandDraft,
    resolved_args: &[String],
) -> std::result::Result<(), String> {
    if !matches!(draft.template.tool, TemplateTool::Forge) {
        return Ok(());
    }
    if !resolved_args
        .first()
        .is_some_and(|command| command.eq_ignore_ascii_case("script"))
    {
        return Ok(());
    }
    if !resolved_args.iter().any(|token| token == "--broadcast") {
        return Ok(());
    }

    let mut private_key_value_indexes = std::collections::BTreeSet::new();
    for (index, token) in resolved_args.iter().enumerate() {
        if token == "--private-key" {
            if index + 1 < resolved_args.len() {
                private_key_value_indexes.insert(index + 1);
            }
            continue;
        }
        if token.starts_with("--private-key=") {
            return Ok(());
        }
    }

    let configured_private_key = draft
        .param_values
        .get("deployer_private_key")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    for (index, token) in resolved_args.iter().enumerate() {
        if private_key_value_indexes.contains(&index) {
            continue;
        }

        let token = token.trim();
        let is_configured_private_key = configured_private_key.is_some_and(|value| value == token);
        if is_configured_private_key || looks_like_private_key_hex(token) {
            return Err(
                "invalid broadcast signer args: use `--private-key <key>`; positional values after `--sig` are function args"
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn looks_like_private_key_hex(value: &str) -> bool {
    value.len() == 66
        && value.starts_with("0x")
        && value[2..].chars().all(|ch| ch.is_ascii_hexdigit())
}

fn extract_rpc_url_from_args(resolved_args: &[String]) -> Option<String> {
    let mut index = 0usize;
    while index < resolved_args.len() {
        let token = resolved_args[index].trim();
        if token == "--rpc-url" {
            return resolved_args
                .get(index + 1)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
        }
        if let Some(value) = token.strip_prefix("--rpc-url=") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use foundry_tui_config::{AppConfig, CustomTemplate, TemplateLoadState, TemplateTool};

    use crate::model::{AnvilInstance, AnvilInstanceStatus};

    use super::AppController;

    #[test]
    fn parse_pasted_template_converts_angle_placeholders() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        let parsed = controller
            .parse_pasted_template(
                "forge script script/Increment.s.sol:IncrementScript \\\n\
                 --rpc-url http://127.0.0.1:8545 \\\n\
                 --broadcast \\\n\
                 --sig \"run(uint256,address)\" \\\n\
                 0xabc \\\n\
                 <CONTRACT_ADDRESS> \\\n\
                 -vv",
            )
            .expect("expected parsed template");

        assert_eq!(parsed.tool, TemplateTool::Forge);
        assert!(parsed
            .args_template
            .contains(&"{{contract_address}}".to_string()));
        assert!(parsed.params.contains_key("contract_address"));
    }

    #[test]
    fn parse_pasted_template_accepts_cast_commands() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        let parsed = controller
            .parse_pasted_template(
                "cast call <CONTRACT_ADDRESS> \"number()(uint256)\" --rpc-url http://127.0.0.1:8545",
            )
            .expect("expected parsed cast template");

        assert_eq!(parsed.tool, TemplateTool::Cast);
        assert!(parsed
            .args_template
            .contains(&"{{contract_address}}".to_string()));
        assert!(parsed.params.contains_key("contract_address"));
    }

    #[test]
    fn new_with_templates_keeps_cast_templates_for_builder() {
        let config = AppConfig::default();
        let controller = AppController::new_with_templates(
            config,
            PathBuf::from("."),
            PathBuf::from("cfg"),
            TemplateLoadState {
                templates: vec![
                    CustomTemplate {
                        id: "cast-call".to_string(),
                        label: "Cast Call".to_string(),
                        tool: TemplateTool::Cast,
                        args_template: vec![
                            "call".to_string(),
                            "{{target}}".to_string(),
                            "{{signature}}".to_string(),
                        ],
                        description: None,
                        tags: Vec::new(),
                        default_rpc_preset: Some("local".to_string()),
                        params: BTreeMap::new(),
                    },
                    CustomTemplate {
                        id: "forge-build".to_string(),
                        label: "Forge Build".to_string(),
                        tool: TemplateTool::Forge,
                        args_template: vec!["build".to_string()],
                        description: None,
                        tags: Vec::new(),
                        default_rpc_preset: None,
                        params: BTreeMap::new(),
                    },
                ],
                global_path: PathBuf::new(),
                project_path: PathBuf::new(),
            },
        );

        assert!(controller
            .model
            .custom_templates
            .iter()
            .any(|template| template.tool == TemplateTool::Cast));
        assert!(controller
            .model
            .custom_templates
            .iter()
            .any(|template| template.tool == TemplateTool::Forge));
    }

    #[test]
    fn build_custom_preview_masks_secret_values() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));

        let template = CustomTemplate {
            id: "test-template".to_string(),
            label: "Test".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "{{deployer_private_key}}".to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let mut draft = controller.new_custom_draft(template);
        draft.merged_args = draft.template.args_template.clone();
        draft.param_values.insert(
            "script_target".to_string(),
            "script/Deploy.s.sol:Deploy".to_string(),
        );
        draft.param_values.insert(
            "deployer_private_key".to_string(),
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        );

        controller
            .build_custom_preview(&mut draft)
            .expect("expected preview");
        assert!(
            draft.display_commandline.contains("******"),
            "display command should mask secrets"
        );
        assert!(
            !draft
                .display_commandline
                .contains("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
            "display command should not include private key"
        );
    }

    #[test]
    fn build_custom_preview_redacts_secret_like_raw_args() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));

        let template = CustomTemplate {
            id: "raw-secret".to_string(),
            label: "Raw Secret".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "script/Deploy.s.sol:Deploy".to_string(),
                "--private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                    .to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let mut draft = controller.new_custom_draft(template);
        draft.merged_args = draft.template.args_template.clone();

        controller
            .build_custom_preview(&mut draft)
            .expect("expected preview");
        assert!(
            draft.display_commandline.contains("--private-key=******"),
            "display command should redact private key flag"
        );
        assert!(
            !draft
                .display_commandline
                .contains("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
            "display command should not include raw private key literal"
        );
    }

    #[test]
    fn build_custom_preview_rejects_broadcast_positional_private_key() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));

        let template = CustomTemplate {
            id: "broadcast-positional-key".to_string(),
            label: "Broadcast Positional Key".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "--rpc-url".to_string(),
                "{{rpc_url}}".to_string(),
                "--broadcast".to_string(),
                "--sig".to_string(),
                "run(address)".to_string(),
                "{{deployer_private_key}}".to_string(),
                "{{contract_address}}".to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let mut draft = controller.new_custom_draft(template);
        draft.merged_args = draft.template.args_template.clone();
        draft.param_values.insert(
            "script_target".to_string(),
            "script/Increment.s.sol:IncrementScript".to_string(),
        );
        draft
            .param_values
            .insert("rpc_url".to_string(), "http://127.0.0.1:8545".to_string());
        draft.param_values.insert(
            "deployer_private_key".to_string(),
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        );
        draft.param_values.insert(
            "contract_address".to_string(),
            "0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496".to_string(),
        );

        let error = controller
            .build_custom_preview(&mut draft)
            .expect_err("expected positional private key to be rejected");
        assert!(
            error.contains("use `--private-key <key>`"),
            "error should direct user to --private-key usage"
        );
    }

    #[test]
    fn build_custom_preview_accepts_broadcast_private_key_flag() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));

        let template = CustomTemplate {
            id: "broadcast-private-key-flag".to_string(),
            label: "Broadcast Private Key Flag".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "--rpc-url".to_string(),
                "{{rpc_url}}".to_string(),
                "--broadcast".to_string(),
                "--sig".to_string(),
                "run(address)".to_string(),
                "--private-key".to_string(),
                "{{deployer_private_key}}".to_string(),
                "{{contract_address}}".to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let mut draft = controller.new_custom_draft(template);
        draft.merged_args = draft.template.args_template.clone();
        draft.param_values.insert(
            "script_target".to_string(),
            "script/Increment.s.sol:IncrementScript".to_string(),
        );
        draft
            .param_values
            .insert("rpc_url".to_string(), "http://127.0.0.1:8545".to_string());
        draft.param_values.insert(
            "deployer_private_key".to_string(),
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        );
        draft.param_values.insert(
            "contract_address".to_string(),
            "0x7FA9385bE102ac3EAc297483Dd6233D62b3e1496".to_string(),
        );

        controller
            .build_custom_preview(&mut draft)
            .expect("expected --private-key flag flow to pass");
        assert!(
            draft.resolved_args.windows(2).any(|pair| {
                pair[0] == "--private-key"
                    && pair[1]
                        == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            }),
            "resolved args should preserve --private-key flag + value"
        );
    }

    #[test]
    fn new_custom_draft_prefers_selected_running_anvil_rpc_url() {
        let config = AppConfig::default();
        let mut controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        controller.model.anvil_instances.push(AnvilInstance {
            job_id: 42,
            name: "anvil-2".to_string(),
            port: 8542,
            fork_url: None,
            status: AnvilInstanceStatus::Running,
            logs: Vec::new(),
        });
        controller.model.selected_anvil_index = 0;

        let template = CustomTemplate {
            id: "selected-anvil-rpc-default".to_string(),
            label: "Selected Anvil RPC Default".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "--rpc-url".to_string(),
                "{{rpc_url}}".to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let draft = controller.new_custom_draft(template);
        assert_eq!(draft.rpc_url.as_deref(), Some("http://127.0.0.1:8542"));
        assert_eq!(
            draft.param_values.get("rpc_url").map(String::as_str),
            Some("http://127.0.0.1:8542")
        );
    }

    #[test]
    fn build_custom_preview_sets_rpc_target_from_resolved_rpc_url_flag() {
        let config = AppConfig::default();
        let controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));

        let template = CustomTemplate {
            id: "resolved-rpc-url-target".to_string(),
            label: "Resolved RPC URL Target".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "--rpc-url".to_string(),
                "{{rpc_url}}".to_string(),
            ],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: Some("local".to_string()),
            params: BTreeMap::new(),
        };

        let mut draft = controller.new_custom_draft(template);
        draft.merged_args = draft.template.args_template.clone();
        draft.param_values.insert(
            "script_target".to_string(),
            "script/Increment.s.sol:IncrementScript".to_string(),
        );
        draft
            .param_values
            .insert("rpc_url".to_string(), "http://127.0.0.1:9555".to_string());

        controller
            .build_custom_preview(&mut draft)
            .expect("expected preview");
        assert_eq!(draft.rpc_url.as_deref(), Some("http://127.0.0.1:9555"));
    }
}
