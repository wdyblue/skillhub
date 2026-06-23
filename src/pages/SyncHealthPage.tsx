import { RefreshCw, Wrench } from "lucide-react";
import { SyncReport } from "../lib/tauri";

type Props = {
  report: SyncReport | null;
  onCheck: () => Promise<void>;
  onFix: () => Promise<void>;
};

export function SyncHealthPage({ report, onCheck, onFix }: Props) {
  async function confirmFix() {
    if (!report || report.needs_fix_count === 0) return;
    const ok = window.confirm(
      `将修复 ${report.needs_fix_count} 个可自动处理的问题。\n\n只会创建或删除软链接，不会删除真实 skill 文件夹。是否继续？`
    );
    if (ok) await onFix();
  }

  const cards = [
    ["正常链接", report?.normal_count ?? 0],
    ["缺失链接", report?.missing_count ?? 0],
    ["断链", report?.broken_count ?? 0],
    ["错链/多余链接", report?.wrong_target_count ?? 0],
    ["重复", report?.duplicate_count ?? 0],
    ["目录不存在", report?.missing_dir_count ?? 0],
    ["需要修复", report?.needs_fix_count ?? 0]
  ];

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">同步体检</h3>
            <p className="mt-2 text-sm text-slate-500">
              检查主仓库与各工具目录之间的软链接状态。修复操作只处理软链接。
              这里不评估 skill 内容质量，内容完整性请看“技能体检”。
            </p>
          </div>
          <div className="flex gap-2">
            <button className="action-button" onClick={() => void onCheck()}>
              <RefreshCw className="h-4 w-4" />
              重新检查
            </button>
            <button className="action-button" onClick={() => void confirmFix()}>
              <Wrench className="h-4 w-4" />
              一键修复
            </button>
          </div>
        </div>

        <div className="mt-5 grid grid-cols-7 gap-3">
          {cards.map(([label, value]) => (
            <div key={label} className="rounded-3xl bg-slate-50 p-4">
              <p className="text-xs text-slate-500">{label}</p>
              <p className="mt-2 text-2xl font-semibold text-slate-950">{value}</p>
            </div>
          ))}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">异常项</h3>
        <div className="mt-4 overflow-hidden rounded-3xl border border-slate-200">
          {!report || report.issues.length === 0 ? (
            <div className="px-5 py-12 text-center text-sm text-slate-500">
              暂无异常。点击“重新检查”获取最新同步状态。
            </div>
          ) : (
            <table className="w-full text-left text-sm">
              <thead className="bg-slate-50 text-xs text-slate-500">
                <tr>
                  <th className="px-4 py-3">Skill</th>
                  <th className="px-4 py-3">工具</th>
                  <th className="px-4 py-3">问题</th>
                  <th className="px-4 py-3">当前路径</th>
                  <th className="px-4 py-3">期望路径</th>
                  <th className="px-4 py-3">建议</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {report.issues.map((issue) => (
                  <tr key={issue.id}>
                    <td className="px-4 py-3">{issue.skill_name ?? "-"}</td>
                    <td className="px-4 py-3">{issue.tool_name}</td>
                    <td className="px-4 py-3">{issue.issue_type}</td>
                    <td className="max-w-xs truncate px-4 py-3">{issue.current_path || "-"}</td>
                    <td className="max-w-xs truncate px-4 py-3">{issue.expected_path || "-"}</td>
                    <td className="px-4 py-3">{issue.fixable ? "可自动修复" : issue.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>
    </div>
  );
}
