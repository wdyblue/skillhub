import { Archive, Database, FolderSearch, Gauge, Layers3, ShieldAlert } from "lucide-react";
import { AppStats, ScanRoot, Skill } from "../lib/tauri";

type Props = {
  loading: boolean;
  stats: AppStats | null;
  skills: Skill[];
  scanRoots: ScanRoot[];
  onScan: () => void;
  onGoSkills: () => void;
  onGoSettings: () => void;
};

export function DashboardPage({
  loading,
  stats,
  skills,
  scanRoots,
  onScan,
  onGoSkills,
  onGoSettings
}: Props) {
  const cards = [
    { label: "总技能数", value: stats?.total_skills ?? 0, icon: Layers3 },
    { label: "未分类", value: stats?.uncategorized_skills ?? 0, icon: FolderSearch },
    { label: "疑似重复", value: stats?.duplicate_risk_skills ?? 0, icon: ShieldAlert },
    { label: "已归档", value: stats?.archived_skills ?? 0, icon: Archive },
    { label: "路径丢失", value: stats?.missing_skills ?? 0, icon: Gauge },
    { label: "扫描目录", value: stats?.scan_roots ?? 0, icon: Database }
  ];

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-[1.5fr_1fr] gap-5">
        <div className="rounded-[2rem] border border-slate-200 bg-white p-7 shadow-soft">
          <p className="text-sm font-medium text-brand-600">本地优先，不联网，不删除</p>
          <h3 className="mt-3 max-w-3xl text-3xl font-semibold tracking-tight text-slate-950">
            把散落在本机的 AI Skills 扫描成可检索、可评分、可归档的资产库。
          </h3>
          <p className="mt-4 max-w-2xl text-sm leading-6 text-slate-600">
            第一版聚焦本地目录管理、SKILL.md 扫描入库、卡片浏览和基础详情。
            所有危险操作默认只改数据库状态，不移动、不删除原始文件。
          </p>
          <div className="mt-6 flex gap-3">
            <button
              onClick={onScan}
              className="rounded-2xl bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 active:scale-[0.98]"
            >
              扫描本地目录
            </button>
            <button
              onClick={onGoSettings}
              className="rounded-2xl border border-slate-200 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 active:scale-[0.98]"
            >
              添加目录
            </button>
          </div>
        </div>

        <div className="rounded-[2rem] border border-slate-200 bg-slate-900 p-7 text-white shadow-soft">
          <p className="text-sm text-slate-300">体检摘要</p>
          <p className="mt-4 text-4xl font-semibold">{stats?.total_skills ?? 0}</p>
          <p className="mt-2 text-sm text-slate-300">已登记技能</p>
          <div className="mt-6 rounded-2xl bg-white/10 p-4 text-sm leading-6 text-slate-200">
            本地共有 {stats?.total_skills ?? 0} 个 skills，其中{" "}
            {stats?.uncategorized_skills ?? 0} 个未分类，
            {stats?.duplicate_risk_skills ?? 0} 个疑似重复，
            {stats?.archived_skills ?? 0} 个已归档。
          </div>
        </div>
      </div>

      <div className="grid grid-cols-6 gap-4">
        {cards.map((card) => {
          const Icon = card.icon;
          return (
            <div
              key={card.label}
              className="rounded-3xl border border-slate-200 bg-white p-4 shadow-sm"
            >
              <Icon className="h-5 w-5 text-brand-600" />
              <p className="mt-4 text-2xl font-semibold text-slate-950">
                {loading ? "..." : card.value}
              </p>
              <p className="mt-1 text-sm text-slate-500">{card.label}</p>
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-[1fr_1fr] gap-5">
        <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-slate-950">最近扫描目录</h3>
            <button className="text-sm font-medium text-brand-600" onClick={onGoSettings}>
              管理
            </button>
          </div>
          <div className="mt-4 space-y-3">
            {scanRoots.length === 0 ? (
              <EmptyText text="还没有添加 skill 根目录。" />
            ) : (
              scanRoots.slice(0, 5).map((root) => (
                <div
                  key={root.id}
                  className="rounded-2xl border border-slate-100 bg-slate-50 px-4 py-3"
                >
                  <p className="truncate text-sm font-medium text-slate-800">{root.path}</p>
                  <p className="mt-1 text-xs text-slate-500">
                    {root.enabled ? "已启用" : "已禁用"} · {root.platform} · 上次扫描：
                    {root.last_scanned_at ?? "尚未扫描"}
                  </p>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-slate-950">推荐优先整理</h3>
            <button className="text-sm font-medium text-brand-600" onClick={onGoSkills}>
              查看全部
            </button>
          </div>
          <div className="mt-4 space-y-3">
            {skills.length === 0 ? (
              <EmptyText text="扫描后会显示低评分、未分类或疑似重复的技能。" />
            ) : (
              skills
                .slice()
                .sort((a, b) => a.quality_score - b.quality_score)
                .slice(0, 5)
                .map((skill) => (
                  <div
                    key={skill.id}
                    className="rounded-2xl border border-slate-100 bg-slate-50 px-4 py-3"
                  >
                    <p className="truncate text-sm font-medium text-slate-800">
                      {skill.name}
                    </p>
                    <p className="mt-1 text-xs text-slate-500">
                      评分 {skill.quality_score} · {skill.category_name ?? "未分类"} ·{" "}
                      {skill.status}
                    </p>
                  </div>
                ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function EmptyText({ text }: { text: string }) {
  return (
    <div className="rounded-2xl border border-dashed border-slate-200 bg-slate-50 px-4 py-8 text-center text-sm text-slate-500">
      {text}
    </div>
  );
}
