import { invoke, isTauri } from "@tauri-apps/api/core";

const TAURI_RUNTIME_ERROR =
  "SkillHub 需要在 Tauri 桌面端中运行。请使用 `npm run tauri:dev` 打开桌面窗口，不要直接在浏览器里访问开发地址。";

function ensureTauriRuntime() {
  if (!isTauri()) {
    throw new Error(TAURI_RUNTIME_ERROR);
  }
}

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>) {
  ensureTauriRuntime();
  return invoke<T>(command, args);
}

export type AppStats = {
  total_skills: number;
  active_skills: number;
  archived_skills: number;
  missing_skills: number;
  uncategorized_skills: number;
  duplicate_risk_skills: number;
  scan_roots: number;
};

export type Category = {
  id: number;
  name: string;
  name_en: string;
  color: string;
  parent_id: number | null;
};

export type Tag = {
  id: number;
  name: string;
  skill_count: number;
};

export type ScanRoot = {
  id: number;
  path: string;
  enabled: boolean;
  platform: string;
  last_scanned_at: string | null;
  created_at: string;
};

export type Skill = {
  id: number;
  name: string;
  path: string;
  description: string;
  content: string;
  name_zh: string;
  name_en: string;
  description_zh: string;
  description_en: string;
  summary_zh: string;
  summary_en: string;
  category_id: number | null;
  category_name: string | null;
  category_color: string | null;
  source: string;
  platform: string;
  scope: string;
  projectPath: string;
  is_custom: boolean;
  status: string;
  quality_score: number;
  quality_reason: string;
  usage_count: number;
  duplicate_score: number;
  archive_recommendation: string;
  classification_confidence: number;
  hash: string;
  created_at: string;
  updated_at: string;
  last_scanned_at: string | null;
  last_used_at: string | null;
  tags: string[];
  tool_links: SkillToolLink[];
};

export type ToolConfig = {
  id: number;
  tool_name: string;
  display_name: string;
  skill_dir: string;
  detected: boolean;
  enabled: boolean;
  sync_enabled: boolean;
  is_custom: boolean;
  linkMode: string;
  last_checked_at: string | null;
};

export type SkillToolLink = {
  id: number;
  skill_id: number;
  tool_name: string;
  enabled: boolean;
  link_path: string;
  linkMode: string;
  link_status: string;
  last_synced_at: string | null;
  error_message: string;
};

export type RepositoryConfig = {
  id: number;
  name: string;
  path: string;
  repo_type: string;
  enabled: boolean;
  is_primary: boolean;
  last_scanned_at: string | null;
};

export type SyncIssue = {
  id: number;
  skill_id: number | null;
  skill_name: string | null;
  tool_name: string;
  issue_type: string;
  current_path: string;
  expected_path: string;
  severity: string;
  fixable: boolean;
  status: string;
  message: string;
};

export type SyncReport = {
  normal_count: number;
  missing_count: number;
  broken_count: number;
  wrong_target_count: number;
  duplicate_count: number;
  missing_dir_count: number;
  needs_fix_count: number;
  issues: SyncIssue[];
};

export type AiConfig = {
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
  apiKey: string;
};

export type Account = {
  loggedIn: boolean;
  name: string;
  email: string;
  avatarInitial: string;
};

export type MarketplaceSource = {
  id: number;
  name: string;
  url: string;
  enabled: boolean;
  lastRefreshedAt: string | null;
};

export type MarketplaceItem = {
  id: number;
  sourceId: number;
  externalId: string;
  name: string;
  description: string;
  version: string;
  author: string;
  category: string;
  tags: string[];
  skillUrl: string;
  homepage: string;
  installedSkillId: number | null;
  installedSkillPath: string;
  installedVersion: string;
  installedHash: string;
  installedAt: string | null;
  lastInstallCheckAt: string | null;
  installStatus: string;
  installMessage: string;
  isUpdateAvailable: boolean;
};

export type CloudSyncConfig = {
  provider: string;
  gistId: string;
  hasToken: boolean;
  lastSyncedAt: string | null;
  accountName: string;
  accountEmail: string;
};

export type SkillListFilters = {
  query?: string;
  categoryId?: number | null;
  tag?: string;
  status?: string;
  source?: string;
  scope?: string;
  onlyArchived?: boolean;
  onlyDuplicate?: boolean;
  onlyUncategorized?: boolean;
  sortBy?: "updated_at" | "quality_score" | "usage_count" | "name";
  sortOrder?: "asc" | "desc";
};

export type ScanSummary = {
  total_found: number;
  new_count: number;
  changed_count: number;
  duplicate_count: number;
  missing_count: number;
};

export function getStats() {
  return tauriInvoke<AppStats>("get_stats");
}

export function listCategories() {
  return tauriInvoke<Category[]>("list_categories");
}

export function listTags() {
  return tauriInvoke<Tag[]>("list_tags");
}

export function createCategory(input: {
  name: string;
  nameEn?: string;
  color?: string;
}) {
  return tauriInvoke<Category>("create_category", { input });
}

export function updateCategory(
  id: number,
  input: { name: string; nameEn?: string; color?: string }
) {
  return tauriInvoke<Category>("update_category", { id, input });
}

export function deleteCategory(id: number) {
  return tauriInvoke<void>("delete_category", { id });
}

export function listScanRoots() {
  return tauriInvoke<ScanRoot[]>("list_scan_roots");
}

export function addScanRoot(path: string, platform: string) {
  return tauriInvoke<ScanRoot>("add_scan_root", { path, platform });
}

export function removeScanRoot(id: number) {
  return tauriInvoke<void>("remove_scan_root", { id });
}

export function toggleScanRoot(id: number, enabled: boolean) {
  return tauriInvoke<void>("toggle_scan_root", { id, enabled });
}

export function scanAll() {
  return tauriInvoke<ScanSummary>("scan_all");
}

export function listSkills(filters: SkillListFilters) {
  return tauriInvoke<Skill[]>("list_skills", { filters });
}

export function getSkill(id: number) {
  return tauriInvoke<Skill>("get_skill", { id });
}

export function updateSkillMeta(
  id: number,
  categoryId: number | null,
  status: string,
  isCustom: boolean
) {
  return tauriInvoke<void>("update_skill_meta", {
    id,
    categoryId,
    status,
    isCustom
  });
}

export function updateSkillScope(request: { id: number; scope: string; projectPath?: string }) {
  return tauriInvoke<void>("update_skill_scope", { request });
}

export function updateSkillTags(request: { skillId: number; tags: string[] }) {
  return tauriInvoke<void>("update_skill_tags", { request });
}

export function batchUpdateSkills(request: {
  skillIds: number[];
  categoryId?: number | null;
  status?: string;
  isCustom?: boolean;
  scope?: string;
  projectPath?: string;
}) {
  return tauriInvoke<void>("batch_update_skills", { request });
}

export function incrementUsage(id: number) {
  return tauriInvoke<void>("increment_usage", { id });
}

export function openSkillFolder(id: number) {
  return tauriInvoke<void>("open_skill_folder", { id });
}

export function openSkillFile(id: number) {
  return tauriInvoke<void>("open_skill_file", { id });
}

export function detectTools() {
  return tauriInvoke<ToolConfig[]>("detect_tools");
}

export function listTools() {
  return tauriInvoke<ToolConfig[]>("list_tools");
}

export function updateToolConfig(request: {
  toolName: string;
  skillDir: string;
  enabled: boolean;
  syncEnabled: boolean;
  linkMode?: string;
}) {
  return tauriInvoke<void>("update_tool_config", { request });
}

export function createCustomTool(request: {
  toolName: string;
  displayName: string;
  skillDir: string;
  linkMode?: string;
}) {
  return tauriInvoke<ToolConfig>("create_custom_tool", { request });
}

export function deleteCustomTool(toolName: string) {
  return tauriInvoke<void>("delete_custom_tool", { toolName });
}

export function listRepositories() {
  return tauriInvoke<RepositoryConfig[]>("list_repositories");
}

export function setPrimaryRepository(path: string) {
  return tauriInvoke<RepositoryConfig>("set_primary_repository", { path });
}

export function setSkillToolEnabled(skillId: number, toolName: string, enabled: boolean) {
  return tauriInvoke<SkillToolLink>("set_skill_tool_enabled", {
    skillId,
    toolName,
    enabled
  });
}

export function checkSyncStatus() {
  return tauriInvoke<SyncReport>("check_sync_status");
}

export function fixSyncIssues() {
  return tauriInvoke<SyncReport>("fix_sync_issues");
}

export function syncAllEnabledTools() {
  return tauriInvoke<SyncReport>("sync_all_enabled_tools");
}

export function createSkillInRepository(request: { name: string; description?: string }) {
  return tauriInvoke<number>("create_skill_in_repository", { request });
}

export function importSkillToRepository(path: string) {
  return tauriInvoke<number>("import_skill_to_repository", { path });
}

export function saveSkillContent(skillId: number, content: string) {
  return tauriInvoke<void>("save_skill_content", { skillId, content });
}

export function getAiConfig(revealSecret = false) {
  return tauriInvoke<AiConfig>("get_ai_config", { revealSecret });
}

export function saveAiConfig(input: {
  baseUrl: string;
  apiKey?: string;
  model: string;
}) {
  return tauriInvoke<AiConfig>("save_ai_config", { input });
}

export function listAiModels() {
  return tauriInvoke<string[]>("list_ai_models");
}

export function testAiConnection() {
  return tauriInvoke<string>("test_ai_connection");
}

export function translateSkill(skillId: number, targetLanguage: "zh" | "en") {
  return tauriInvoke<Skill>("translate_skill", {
    input: { skillId, targetLanguage }
  });
}

export function clearTranslationCache() {
  return tauriInvoke<void>("clear_translation_cache");
}

export function getAccount() {
  return tauriInvoke<Account>("get_account");
}

export function loginAccount(input: { name: string; email?: string }) {
  return tauriInvoke<Account>("login_account", { input });
}

export function logoutAccount() {
  return tauriInvoke<Account>("logout_account");
}

export function listMarketplaceSources() {
  return tauriInvoke<MarketplaceSource[]>("list_marketplace_sources");
}

export function addMarketplaceSource(input: { name: string; url: string }) {
  return tauriInvoke<MarketplaceSource>("add_marketplace_source", { input });
}

export function deleteMarketplaceSource(id: number) {
  return tauriInvoke<void>("delete_marketplace_source", { id });
}

export function refreshMarketplaceSource(sourceId: number) {
  return tauriInvoke<MarketplaceItem[]>("refresh_marketplace_source", { sourceId });
}

export function listMarketplaceItems() {
  return tauriInvoke<MarketplaceItem[]>("list_marketplace_items");
}

export function installMarketplaceItem(itemId: number) {
  return tauriInvoke<number>("install_marketplace_item", { itemId });
}

export function updateMarketplaceItem(itemId: number) {
  return tauriInvoke<number>("update_marketplace_item", { itemId });
}

export function uninstallMarketplaceItem(itemId: number) {
  return tauriInvoke<void>("uninstall_marketplace_item", { itemId });
}

export function recheckMarketplaceInstallations() {
  return tauriInvoke<MarketplaceItem[]>("recheck_marketplace_installations");
}

export function exportSyncPackage(path: string) {
  return tauriInvoke<void>("export_sync_package", { path });
}

export function importSyncPackage(path: string) {
  return tauriInvoke<void>("import_sync_package", { path });
}

export function getCloudSyncConfig() {
  return tauriInvoke<CloudSyncConfig>("get_cloud_sync_config");
}

export function saveCloudSyncConfig(input: { gistId: string; token?: string }) {
  return tauriInvoke<CloudSyncConfig>("save_cloud_sync_config", { input });
}

export function pushSyncPackageToCloud() {
  return tauriInvoke<string>("push_sync_package_to_cloud");
}

export function pullSyncPackageFromCloud() {
  return tauriInvoke<string>("pull_sync_package_from_cloud");
}
