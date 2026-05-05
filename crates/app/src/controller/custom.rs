use std::collections::BTreeMap;

use chrono::Local;
use foundry_tui_config::{CustomTemplate, TemplateTool};
use foundry_tui_foundry::{redact_cli_args, ToolEvent, ToolKind, ToolRequest};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{CustomCommandDraft, CustomCommandModal, CustomModalStep, Tab},
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
            Some("forge builder opened: pick a preset or paste a forge command".to_string());
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
            return Err("paste a full forge command, for example `forge script ...`".to_string());
        }

        let binary = tokens.remove(0).to_lowercase();
        let tool = parse_template_tool(&binary)
            .ok_or_else(|| format!("unsupported command `{binary}` (use forge)"))?;
        if !matches!(tool, TemplateTool::Forge) {
            return Err("forge builder currently accepts only `forge ...` commands".to_string());
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
        let rpc_url = self.rpc_url_for_preset(&rpc_preset);
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
        if !extract_placeholders(&draft.template.args_template)
            .iter()
            .any(|placeholder| placeholder == "rpc_url")
        {
            return;
        }

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

        if !should_replace {
            return;
        }

        if let Some(next_rpc_url) = self.rpc_url_for_preset(&draft.rpc_preset) {
            draft
                .param_values
                .insert("rpc_url".to_string(), next_rpc_url);
        }
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
        let rpc_url = self.rpc_url_for_preset(&draft.rpc_preset);
        if let Some(url) = rpc_url.clone() {
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
            return Err("forge command cannot be empty".to_string());
        }

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
        draft.rpc_url = rpc_url;

        Ok(())
    }

    pub(crate) fn run_custom_draft(
        &mut self,
        draft: &CustomCommandDraft,
        tool_events: &UnboundedSender<ToolEvent>,
    ) -> std::result::Result<u64, String> {
        if !matches!(draft.template.tool, TemplateTool::Forge) {
            return Err("forge builder supports only forge presets".to_string());
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

        let label = format!("Forge Builder: {}", draft.template.label);
        let job_id = self.start_tool_request_job(&label, request, tool_events)?;
        self.model.notification = Some(format!("queued forge builder command as job #{job_id}"));
        Ok(job_id)
    }

    pub(crate) fn rpc_url_for_preset(&self, preset: &str) -> Option<String> {
        self.config.rpc_presets.get(preset).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use foundry_tui_config::{AppConfig, CustomTemplate, TemplateTool};

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
                 <COUNTER_ADDR> \\\n\
                 -vv",
            )
            .expect("expected parsed template");

        assert_eq!(parsed.tool, TemplateTool::Forge);
        assert!(parsed
            .args_template
            .contains(&"{{counter_addr}}".to_string()));
        assert!(parsed.params.contains_key("counter_addr"));
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
}
