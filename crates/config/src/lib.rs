use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionId {
    Quit,
    NextTab,
    PrevTab,
    FocusNextSection,
    FocusPrevSection,
    OpenPalette,
    OpenThemePicker,
    ThemeNext,
    ThemePrev,
    RunCustomCommand,
    RunBuild,
    RunTest,
    RunScript,
    RunCastBlockNumber,
    RunVerifyCheck,
    RunChiselList,
    RunFoundryupUpdate,
    StartAnvil,
    StopAnvil,
    ScrollLogsUp,
    ScrollLogsDown,
}

impl ActionId {
    pub fn label(self) -> &'static str {
        match self {
            ActionId::Quit => "Quit",
            ActionId::NextTab => "Next Tab",
            ActionId::PrevTab => "Previous Tab",
            ActionId::FocusNextSection => "Focus Next Section",
            ActionId::FocusPrevSection => "Focus Previous Section",
            ActionId::OpenPalette => "Command Palette",
            ActionId::OpenThemePicker => "Legacy Theme Picker",
            ActionId::ThemeNext => "Legacy Theme Next",
            ActionId::ThemePrev => "Legacy Theme Previous",
            ActionId::RunCustomCommand => "Forge Command Builder",
            ActionId::RunBuild => "Forge Build",
            ActionId::RunTest => "Forge Test",
            ActionId::RunScript => "Forge Script",
            ActionId::RunCastBlockNumber => "Cast Block Number",
            ActionId::RunVerifyCheck => "Forge Verify Check",
            ActionId::RunChiselList => "Chisel List",
            ActionId::RunFoundryupUpdate => "Foundryup Update",
            ActionId::StartAnvil => "Start Anvil",
            ActionId::StopAnvil => "Stop Anvil",
            ActionId::ScrollLogsUp => "Scroll Section Up",
            ActionId::ScrollLogsDown => "Scroll Section Down",
        }
    }

    pub fn palette_defaults() -> Vec<ActionId> {
        vec![
            ActionId::RunCustomCommand,
            ActionId::RunBuild,
            ActionId::RunTest,
            ActionId::RunScript,
            ActionId::RunChiselList,
            ActionId::StartAnvil,
            ActionId::StopAnvil,
            ActionId::RunCastBlockNumber,
            ActionId::RunVerifyCheck,
            ActionId::RunFoundryupUpdate,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub tick_rate_ms: u64,
    pub frame_rate_ms: u64,
    pub show_welcome: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            frame_rate_ms: 33,
            show_welcome: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub panel: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub muted: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        bold_contrast_theme()
    }
}

fn bold_contrast_theme() -> ThemeConfig {
    ThemeConfig {
        name: "bold-contrast".to_string(),
        background: "#00000E".to_string(),
        foreground: "#FFFFFF".to_string(),
        accent: "#87CEEB".to_string(),
        panel: "#00000E".to_string(),
        success: "#22C55E".to_string(),
        warning: "#FACC15".to_string(),
        danger: "#EF4444".to_string(),
        muted: "#9CA3AF".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyConfig {
    pub bindings: BTreeMap<ActionId, String>,
}

impl Default for KeyConfig {
    fn default() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(ActionId::Quit, "q".to_string());
        bindings.insert(ActionId::NextTab, "tab".to_string());
        bindings.insert(ActionId::PrevTab, "backtab".to_string());
        bindings.insert(ActionId::FocusNextSection, "ctrl+j".to_string());
        bindings.insert(ActionId::FocusPrevSection, "ctrl+k".to_string());
        bindings.insert(ActionId::OpenPalette, "ctrl+p".to_string());
        bindings.insert(ActionId::RunCustomCommand, "x".to_string());
        bindings.insert(ActionId::RunBuild, "b".to_string());
        bindings.insert(ActionId::RunTest, "t".to_string());
        bindings.insert(ActionId::RunScript, "s".to_string());
        bindings.insert(ActionId::RunCastBlockNumber, "c".to_string());
        bindings.insert(ActionId::RunVerifyCheck, "v".to_string());
        bindings.insert(ActionId::RunChiselList, "h".to_string());
        bindings.insert(ActionId::RunFoundryupUpdate, "u".to_string());
        bindings.insert(ActionId::StartAnvil, "a".to_string());
        bindings.insert(ActionId::StopAnvil, "shift+a".to_string());
        bindings.insert(ActionId::ScrollLogsUp, "up".to_string());
        bindings.insert(ActionId::ScrollLogsDown, "down".to_string());
        Self { bindings }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkflowCommands {
    pub build: Vec<String>,
    pub test: Vec<String>,
    pub script: Vec<String>,
    pub cast_block_number: Vec<String>,
    pub verify_check: Vec<String>,
    pub chisel_list: Vec<String>,
    pub foundryup_update: Vec<String>,
    pub anvil_start: Vec<String>,
}

impl Default for WorkflowCommands {
    fn default() -> Self {
        Self {
            build: vec!["build".to_string()],
            test: vec!["test".to_string(), "-vv".to_string()],
            script: vec![
                "script".to_string(),
                "script/Deploy.s.sol:DeployScript".to_string(),
                "--sig".to_string(),
                "run()".to_string(),
            ],
            cast_block_number: vec!["block-number".to_string()],
            verify_check: vec!["verify-check".to_string(), "<GUID>".to_string()],
            chisel_list: vec!["list".to_string()],
            foundryup_update: vec!["--update".to_string()],
            anvil_start: vec!["--port".to_string(), "8545".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FoundryConfig {
    pub profile: String,
    pub project_root: Option<PathBuf>,
    pub default_rpc_preset: Option<String>,
    pub workflows: WorkflowCommands,
}

impl Default for FoundryConfig {
    fn default() -> Self {
        Self {
            profile: "default".to_string(),
            project_root: None,
            default_rpc_preset: Some("across-ethereum-1".to_string()),
            workflows: WorkflowCommands::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JobConfig {
    pub max_concurrent: usize,
    pub keep_history: usize,
    pub max_log_lines: usize,
    pub auto_scroll_logs: bool,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            keep_history: 300,
            max_log_lines: 2_000,
            auto_scroll_logs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub theme: ThemeConfig,
    pub keys: KeyConfig,
    pub foundry: FoundryConfig,
    pub rpc_presets: BTreeMap<String, String>,
    pub jobs: JobConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut rpc_presets = BTreeMap::new();
        rpc_presets.insert("local".to_string(), "http://127.0.0.1:8545".to_string());
        add_across_rpc_presets(&mut rpc_presets);

        Self {
            ui: UiConfig::default(),
            theme: ThemeConfig::default(),
            keys: KeyConfig::default(),
            foundry: FoundryConfig::default(),
            rpc_presets,
            jobs: JobConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigLoadState {
    Loaded(PathBuf),
    Created(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateTool {
    Forge,
    Cast,
    Anvil,
    Chisel,
    Foundryup,
}

impl Default for TemplateTool {
    fn default() -> Self {
        Self::Forge
    }
}

impl TemplateTool {
    pub fn binary(self) -> &'static str {
        match self {
            TemplateTool::Forge => "forge",
            TemplateTool::Cast => "cast",
            TemplateTool::Anvil => "anvil",
            TemplateTool::Chisel => "chisel",
            TemplateTool::Foundryup => "foundryup",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateParamKind {
    String,
    Address,
    Hex,
    Uint,
}

impl Default for TemplateParamKind {
    fn default() -> Self {
        Self::String
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemplateParamMeta {
    pub label: Option<String>,
    pub default: Option<String>,
    pub secret: bool,
    pub optional: bool,
    pub kind: TemplateParamKind,
}

impl Default for TemplateParamMeta {
    fn default() -> Self {
        Self {
            label: None,
            default: None,
            secret: false,
            optional: false,
            kind: TemplateParamKind::String,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTemplate {
    pub id: String,
    pub label: String,
    pub tool: TemplateTool,
    pub args_template: Vec<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub default_rpc_preset: Option<String>,
    pub params: BTreeMap<String, TemplateParamMeta>,
}

impl Default for CustomTemplate {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            tool: TemplateTool::Forge,
            args_template: Vec::new(),
            description: None,
            tags: Vec::new(),
            default_rpc_preset: None,
            params: BTreeMap::new(),
        }
    }
}

impl CustomTemplate {
    fn normalized(mut self) -> Option<Self> {
        self.id = self.id.trim().to_string();
        self.label = self.label.trim().to_string();
        if self.id.is_empty() || self.label.is_empty() {
            return None;
        }

        self.args_template = self
            .args_template
            .into_iter()
            .map(|arg| arg.trim().to_string())
            .filter(|arg| !arg.is_empty())
            .collect();
        if self.args_template.is_empty() {
            return None;
        }

        Some(self)
    }
}

#[derive(Debug, Clone)]
pub struct TemplateLoadState {
    pub templates: Vec<CustomTemplate>,
    pub global_path: PathBuf,
    pub project_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct TemplateFile {
    templates: Vec<CustomTemplate>,
}

impl Default for TemplateFile {
    fn default() -> Self {
        Self {
            templates: default_custom_templates(),
        }
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("failed to resolve config directory")?;
    Ok(base.join("foundry-tui").join("config.toml"))
}

pub fn default_templates_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("failed to resolve config directory")?;
    Ok(base.join("foundry-tui").join("templates.toml"))
}

pub fn default_project_templates_path(project_root: &Path) -> PathBuf {
    project_root.join(".foundry-tui").join("templates.toml")
}

pub fn load_or_create() -> Result<(AppConfig, ConfigLoadState)> {
    let path = default_config_path()?;
    load_or_create_at(&path)
}

pub fn load_or_create_at(path: &Path) -> Result<(AppConfig, ConfigLoadState)> {
    if path.exists() {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;
        let mut config: AppConfig = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file at {}", path.display()))?;
        maybe_upgrade_legacy_theme(&mut config);
        ensure_default_rpc_presets(&mut config);
        ensure_default_key_bindings(&mut config);
        return Ok((config, ConfigLoadState::Loaded(path.to_path_buf())));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create config directory at {}",
                parent.to_string_lossy()
            )
        })?;
    }

    let config = AppConfig::default();
    let contents = toml::to_string_pretty(&config).context("failed to serialize default config")?;
    write_secure_file(path, &contents)
        .with_context(|| format!("failed to write default config at {}", path.display()))?;

    Ok((config, ConfigLoadState::Created(path.to_path_buf())))
}

pub fn load_templates(project_root: &Path) -> Result<TemplateLoadState> {
    let global_path = default_templates_path()?;
    let project_path = default_project_templates_path(project_root);

    let global_templates = load_or_create_template_file(&global_path)?;
    let project_templates = load_template_file_if_exists(&project_path)?;
    let templates = merge_custom_templates(global_templates, project_templates);

    Ok(TemplateLoadState {
        templates,
        global_path,
        project_path,
    })
}

fn load_or_create_template_file(path: &Path) -> Result<Vec<CustomTemplate>> {
    if path.exists() {
        return load_template_file(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create template directory at {}",
                parent.to_string_lossy()
            )
        })?;
    }

    let file = TemplateFile::default();
    let contents = toml::to_string_pretty(&file).context("failed to serialize templates file")?;
    write_secure_file(path, &contents)
        .with_context(|| format!("failed to write default templates at {}", path.display()))?;
    Ok(file
        .templates
        .into_iter()
        .filter_map(CustomTemplate::normalized)
        .collect())
}

fn write_secure_file(path: &Path, contents: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt};

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("failed to create secure file at {}", path.display()))?;
        file.write_all(contents.as_bytes())
            .with_context(|| format!("failed to write secure file at {}", path.display()))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents)
            .with_context(|| format!("failed to write file at {}", path.display()))?;
        Ok(())
    }
}

fn load_template_file_if_exists(path: &Path) -> Result<Vec<CustomTemplate>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    load_template_file(path)
}

fn load_template_file(path: &Path) -> Result<Vec<CustomTemplate>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read template file at {}", path.display()))?;
    let file: TemplateFile = toml::from_str(&contents)
        .with_context(|| format!("failed to parse template file at {}", path.display()))?;
    Ok(file
        .templates
        .into_iter()
        .filter_map(CustomTemplate::normalized)
        .collect())
}

fn merge_custom_templates(
    global_templates: Vec<CustomTemplate>,
    project_templates: Vec<CustomTemplate>,
) -> Vec<CustomTemplate> {
    let mut merged = BTreeMap::new();
    for template in global_templates {
        merged.insert(template.id.clone(), template);
    }
    for template in project_templates {
        merged.insert(template.id.clone(), template);
    }

    let mut ordered = merged.into_values().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.label
            .to_lowercase()
            .cmp(&right.label.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });

    if ordered.is_empty() {
        return default_custom_templates();
    }

    ordered
}

pub fn default_custom_templates() -> Vec<CustomTemplate> {
    let mut build_params = BTreeMap::new();
    build_params.insert(
        "verbosity".to_string(),
        TemplateParamMeta {
            label: Some("Verbosity".to_string()),
            default: Some("-vv".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::String,
        },
    );

    let mut test_params = BTreeMap::new();
    test_params.insert(
        "verbosity".to_string(),
        TemplateParamMeta {
            label: Some("Verbosity".to_string()),
            default: Some("-vv".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::String,
        },
    );

    let mut script_params = BTreeMap::new();
    script_params.insert(
        "script_target".to_string(),
        TemplateParamMeta {
            label: Some("Script Target".to_string()),
            default: Some("script/Increment.s.sol:IncrementScript".to_string()),
            secret: false,
            optional: false,
            kind: TemplateParamKind::String,
        },
    );
    script_params.insert(
        "rpc_url".to_string(),
        TemplateParamMeta {
            label: Some("RPC URL".to_string()),
            default: Some("http://127.0.0.1:8545".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::String,
        },
    );
    script_params.insert(
        "signature".to_string(),
        TemplateParamMeta {
            label: Some("Function Signature".to_string()),
            default: Some("run(uint256,address)".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::String,
        },
    );
    script_params.insert(
        "deployer_private_key".to_string(),
        TemplateParamMeta {
            label: Some("Deployer Private Key".to_string()),
            default: None,
            secret: true,
            optional: false,
            kind: TemplateParamKind::Hex,
        },
    );
    script_params.insert(
        "verbosity".to_string(),
        TemplateParamMeta {
            label: Some("Verbosity".to_string()),
            default: Some("-vv".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::String,
        },
    );

    let mut verify_params = BTreeMap::new();
    verify_params.insert(
        "guid".to_string(),
        TemplateParamMeta {
            label: Some("Verification GUID".to_string()),
            default: Some("<GUID>".to_string()),
            secret: false,
            optional: false,
            kind: TemplateParamKind::String,
        },
    );

    let mut verify_contract_params = BTreeMap::new();
    verify_contract_params.insert(
        "contract_address".to_string(),
        TemplateParamMeta {
            label: Some("Contract Address".to_string()),
            default: Some("0x0000000000000000000000000000000000000000".to_string()),
            secret: false,
            optional: false,
            kind: TemplateParamKind::Address,
        },
    );
    verify_contract_params.insert(
        "contract_identifier".to_string(),
        TemplateParamMeta {
            label: Some("Contract Identifier".to_string()),
            default: Some("src/Counter.sol:Counter".to_string()),
            secret: false,
            optional: false,
            kind: TemplateParamKind::String,
        },
    );
    verify_contract_params.insert(
        "chain_id".to_string(),
        TemplateParamMeta {
            label: Some("Chain ID".to_string()),
            default: Some("1".to_string()),
            secret: false,
            optional: true,
            kind: TemplateParamKind::Uint,
        },
    );
    verify_contract_params.insert(
        "etherscan_api_key".to_string(),
        TemplateParamMeta {
            label: Some("Etherscan API Key".to_string()),
            default: None,
            secret: true,
            optional: false,
            kind: TemplateParamKind::String,
        },
    );

    vec![
        CustomTemplate {
            id: "forge-build".to_string(),
            label: "Forge Build".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec!["build".to_string(), "{{verbosity}}".to_string()],
            description: Some("Build contracts with configurable verbosity.".to_string()),
            tags: vec!["forge".to_string(), "build".to_string()],
            default_rpc_preset: None,
            params: build_params,
        },
        CustomTemplate {
            id: "forge-test".to_string(),
            label: "Forge Test".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec!["test".to_string(), "{{verbosity}}".to_string()],
            description: Some("Run tests with configurable verbosity.".to_string()),
            tags: vec!["forge".to_string(), "test".to_string()],
            default_rpc_preset: None,
            params: test_params,
        },
        CustomTemplate {
            id: "forge-script-broadcast".to_string(),
            label: "Forge Script Broadcast".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "script".to_string(),
                "{{script_target}}".to_string(),
                "--rpc-url".to_string(),
                "{{rpc_url}}".to_string(),
                "--broadcast".to_string(),
                "--sig".to_string(),
                "{{signature}}".to_string(),
                "{{deployer_private_key}}".to_string(),
                "{{verbosity}}".to_string(),
            ],
            description: Some("Broadcast a forge script with runtime args.".to_string()),
            tags: vec![
                "forge".to_string(),
                "script".to_string(),
                "broadcast".to_string(),
            ],
            default_rpc_preset: Some("local".to_string()),
            params: script_params,
        },
        CustomTemplate {
            id: "forge-verify-check".to_string(),
            label: "Forge Verify Check".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec!["verify-check".to_string(), "{{guid}}".to_string()],
            description: Some("Check verification status using a guid.".to_string()),
            tags: vec!["forge".to_string(), "verify".to_string()],
            default_rpc_preset: None,
            params: verify_params,
        },
        CustomTemplate {
            id: "forge-verify-contract".to_string(),
            label: "Forge Verify Contract".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec![
                "verify-contract".to_string(),
                "{{contract_address}}".to_string(),
                "{{contract_identifier}}".to_string(),
                "--chain-id".to_string(),
                "{{chain_id}}".to_string(),
                "--etherscan-api-key".to_string(),
                "{{etherscan_api_key}}".to_string(),
                "--watch".to_string(),
            ],
            description: Some(
                "Verify a deployed contract and watch status until completion.".to_string(),
            ),
            tags: vec![
                "forge".to_string(),
                "verify".to_string(),
                "contract".to_string(),
            ],
            default_rpc_preset: None,
            params: verify_contract_params,
        },
    ]
}

fn add_across_rpc_presets(rpc_presets: &mut BTreeMap<String, String>) {
    rpc_presets.insert(
        "across-ethereum-1".to_string(),
        "https://mainnet.gateway.tenderly.co".to_string(),
    );
    rpc_presets.insert(
        "across-megaeth-4326".to_string(),
        "https://mainnet.megaeth.com/rpc".to_string(),
    );
    rpc_presets.insert(
        "across-optimism-10".to_string(),
        "https://mainnet.optimism.io".to_string(),
    );
    rpc_presets.insert(
        "across-polygon-137".to_string(),
        "https://polygon.drpc.org".to_string(),
    );
    rpc_presets.insert(
        "across-arbitrum-42161".to_string(),
        "https://arb1.arbitrum.io/rpc".to_string(),
    );
    rpc_presets.insert(
        "across-zksync-324".to_string(),
        "https://mainnet.era.zksync.io".to_string(),
    );
    rpc_presets.insert(
        "across-base-8453".to_string(),
        "https://mainnet.base.org".to_string(),
    );
    rpc_presets.insert(
        "across-linea-59144".to_string(),
        "https://rpc.linea.build".to_string(),
    );
    rpc_presets.insert(
        "across-mode-34443".to_string(),
        "https://mainnet.mode.network".to_string(),
    );
    rpc_presets.insert(
        "across-blast-81457".to_string(),
        "https://rpc.blast.io".to_string(),
    );
    rpc_presets.insert(
        "across-lisk-1135".to_string(),
        "https://rpc.api.lisk.com".to_string(),
    );
    rpc_presets.insert(
        "across-zora-7777777".to_string(),
        "https://rpc.zora.energy".to_string(),
    );
    rpc_presets.insert(
        "across-world-chain-480".to_string(),
        "https://worldchain-mainnet.g.alchemy.com/public".to_string(),
    );
    rpc_presets.insert(
        "across-ink-57073".to_string(),
        "https://rpc-gel.inkonchain.com".to_string(),
    );
    rpc_presets.insert(
        "across-soneium-1868".to_string(),
        "https://rpc.soneium.org".to_string(),
    );
    rpc_presets.insert(
        "across-unichain-130".to_string(),
        "https://mainnet.unichain.org".to_string(),
    );
    rpc_presets.insert(
        "across-lens-232".to_string(),
        "https://api.lens.matterhosted.dev".to_string(),
    );
    rpc_presets.insert(
        "across-bnb-smart-chain-56".to_string(),
        "https://bsc-dataseed1.binance.org".to_string(),
    );
    rpc_presets.insert(
        "across-solana-34268394551451".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
    );
    rpc_presets.insert(
        "across-hyperevm-999".to_string(),
        "https://rpc.hyperliquid.xyz/evm".to_string(),
    );
    rpc_presets.insert(
        "across-plasma-9745".to_string(),
        "https://rpc.plasma.to".to_string(),
    );
    rpc_presets.insert(
        "across-monad-143".to_string(),
        "https://rpc-mainnet.monadinfra.com".to_string(),
    );
    rpc_presets.insert(
        "across-tempo-4217".to_string(),
        "https://rpc.tempo.xyz".to_string(),
    );
    rpc_presets.insert(
        "across-hypercore-1337".to_string(),
        "https://api.hyperliquid.xyz".to_string(),
    );
    rpc_presets.insert(
        "across-lighter-2337".to_string(),
        "https://mainnet.zklighter.elliot.ai".to_string(),
    );
}

fn ensure_default_rpc_presets(config: &mut AppConfig) {
    if !config.rpc_presets.contains_key("local") {
        config
            .rpc_presets
            .insert("local".to_string(), "http://127.0.0.1:8545".to_string());
    }

    let mut across_defaults = BTreeMap::new();
    add_across_rpc_presets(&mut across_defaults);
    for (key, value) in across_defaults {
        config.rpc_presets.entry(key).or_insert(value);
    }

    if config.foundry.default_rpc_preset.is_none() {
        config.foundry.default_rpc_preset = Some("across-ethereum-1".to_string());
    }
}

fn ensure_default_key_bindings(config: &mut AppConfig) {
    let default_bindings = KeyConfig::default().bindings;
    for (action, binding) in default_bindings {
        config.keys.bindings.entry(action).or_insert(binding);
    }
}

fn maybe_upgrade_legacy_theme(config: &mut AppConfig) {
    if config.theme == legacy_midnight_forge_theme() || config.theme == legacy_graphite_dusk_theme()
    {
        config.theme = ThemeConfig::default();
    }
}

fn legacy_midnight_forge_theme() -> ThemeConfig {
    ThemeConfig {
        name: "midnight-forge".to_string(),
        background: "#0B1117".to_string(),
        foreground: "#DCE7F5".to_string(),
        accent: "#20C997".to_string(),
        panel: "#13202B".to_string(),
        success: "#22C55E".to_string(),
        warning: "#F59E0B".to_string(),
        danger: "#EF4444".to_string(),
        muted: "#8AA4BF".to_string(),
    }
}

fn legacy_graphite_dusk_theme() -> ThemeConfig {
    ThemeConfig {
        name: "graphite-dusk".to_string(),
        background: "#0F1115".to_string(),
        foreground: "#E6EAF2".to_string(),
        accent: "#7AA2F7".to_string(),
        panel: "#161A21".to_string(),
        success: "#73D0A2".to_string(),
        warning: "#E7B56A".to_string(),
        danger: "#F07A95".to_string(),
        muted: "#95A0B5".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn defaults_have_core_keybindings() {
        let cfg = AppConfig::default();
        assert!(cfg.keys.bindings.contains_key(&ActionId::RunBuild));
        assert!(cfg.keys.bindings.contains_key(&ActionId::RunTest));
        assert!(cfg.keys.bindings.contains_key(&ActionId::RunCustomCommand));
        assert!(cfg.keys.bindings.contains_key(&ActionId::RunChiselList));
        assert!(cfg
            .keys
            .bindings
            .contains_key(&ActionId::RunFoundryupUpdate));
        assert!(cfg.keys.bindings.contains_key(&ActionId::OpenPalette));
        assert!(cfg.keys.bindings.contains_key(&ActionId::FocusNextSection));
        assert!(cfg.keys.bindings.contains_key(&ActionId::FocusPrevSection));
        assert_eq!(
            cfg.foundry.default_rpc_preset.as_deref(),
            Some("across-ethereum-1")
        );
        assert!(cfg.rpc_presets.contains_key("local"));
        assert!(cfg.rpc_presets.contains_key("across-base-8453"));
        assert!(cfg.rpc_presets.contains_key("across-solana-34268394551451"));
        assert!(cfg.rpc_presets.contains_key("across-hypercore-1337"));
    }

    #[test]
    fn legacy_theme_is_upgraded_automatically() {
        let mut cfg = AppConfig {
            theme: legacy_midnight_forge_theme(),
            ..AppConfig::default()
        };
        maybe_upgrade_legacy_theme(&mut cfg);
        assert_eq!(cfg.theme, ThemeConfig::default());
    }

    #[test]
    fn old_default_graphite_theme_is_upgraded_automatically() {
        let mut cfg = AppConfig {
            theme: legacy_graphite_dusk_theme(),
            ..AppConfig::default()
        };
        maybe_upgrade_legacy_theme(&mut cfg);
        assert_eq!(cfg.theme, ThemeConfig::default());
    }

    #[test]
    fn custom_theme_is_not_overwritten() {
        let mut cfg = AppConfig::default();
        cfg.theme.accent = "#FF0000".to_string();
        maybe_upgrade_legacy_theme(&mut cfg);
        assert_eq!(cfg.theme.accent, "#FF0000");
    }

    #[test]
    fn ensure_default_rpc_presets_backfills_loaded_configs() {
        let mut cfg = AppConfig::default();
        cfg.rpc_presets.clear();
        cfg.foundry.default_rpc_preset = None;

        ensure_default_rpc_presets(&mut cfg);

        assert!(cfg.rpc_presets.contains_key("local"));
        assert!(cfg.rpc_presets.contains_key("across-ethereum-1"));
        assert_eq!(
            cfg.foundry.default_rpc_preset.as_deref(),
            Some("across-ethereum-1")
        );
    }

    #[test]
    fn ensure_default_key_bindings_backfills_missing_entries() {
        let mut cfg = AppConfig::default();
        cfg.keys.bindings.remove(&ActionId::FocusNextSection);
        cfg.keys.bindings.remove(&ActionId::FocusPrevSection);

        ensure_default_key_bindings(&mut cfg);

        assert!(cfg.keys.bindings.contains_key(&ActionId::FocusNextSection));
        assert!(cfg.keys.bindings.contains_key(&ActionId::FocusPrevSection));
    }

    #[test]
    fn load_or_create_template_file_bootstraps_defaults() {
        let temp_dir = test_temp_dir();
        let templates_path = temp_dir.join("templates.toml");

        let templates =
            load_or_create_template_file(&templates_path).expect("expected template bootstrap");

        assert!(templates_path.exists());
        assert!(!templates.is_empty());
        assert!(templates
            .iter()
            .any(|template| template.id == "forge-build"));
        assert!(templates
            .iter()
            .any(|template| template.id == "forge-verify-check"));
        assert!(templates
            .iter()
            .any(|template| template.id == "forge-verify-contract"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn merge_custom_templates_prefers_project_on_id_conflict() {
        let global = vec![CustomTemplate {
            id: "shared".to_string(),
            label: "Global Template".to_string(),
            tool: TemplateTool::Forge,
            args_template: vec!["test".to_string()],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: None,
            params: BTreeMap::new(),
        }];

        let project = vec![CustomTemplate {
            id: "shared".to_string(),
            label: "Project Template".to_string(),
            tool: TemplateTool::Cast,
            args_template: vec!["block-number".to_string()],
            description: None,
            tags: Vec::new(),
            default_rpc_preset: None,
            params: BTreeMap::new(),
        }];

        let merged = merge_custom_templates(global, project);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].label, "Project Template");
        assert_eq!(merged[0].tool, TemplateTool::Cast);
    }

    fn test_temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("foundry-tui-config-test-{nonce}"))
    }
}
