import { AlertTriangle, Archive, FileWarning, Gauge } from "lucide-react";
import { Skill } from "../lib/tauri";

type Props = {
  skills: Skill[];
  onSelectSkill: (id: number) => void;
};

export function HealthPage({ skills, onSelectSkill }: Props) {
  const lowScore = skills.filter((skill) => skill.quality_score < 60);
  const review = skills.filter((skill) => skill.archive_recommendation === "建议检查");
  const oldVersion = skills.filter((skill) => skill.archive_recommendation === "疑似旧版本");
  const missing = skills.filter((skill) => skill.status === "路径丢失");
  const noSummary = skills.filter((skill) => !skill.summary_zh && !skill.summary_en && !skill.description);

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">技能体检说明</h3>
        <p className="mt-2 text-sm leading-6 text-slate-500">
          技能体检关注 skill 内容本身：说明是否完整、评分是否偏低、是否疑似旧版本、是否缺少摘要。
          同步体检关注文件系统和工具目录：软链接是否缺失、断链、错链、工具目录是否存在。
          两者问题域不同，建议保留为两个页面。
        </p>
      </section>

      <section className="grid grid-cols-5 gap-4">
        <Card label="低评分" value={lowScore.length} icon={<Gauge className="h-5 w-5" />} />
        <Card label="建议检查" value={review.length} icon={<AlertTriangle className="h-5 w-5" />} />
        <Card label="疑似旧版本" value={oldVersion.length} icon={<Archive className="h-5 w-5" />} />
        <Card label="路径丢失" value={missing.length} icon={<FileWarning className="h-5 w-5" />} />
        <Card label="缺少摘要" value={noSummary.length} icon={<FileWarning className="h-5 w-5" />} />
      </section>

      <IssueSection title="优先处理：低评分 Skill" skills={lowScore} onSelectSkill={onSelectSkill} />
      <IssueSection title="建议检查" skills={review} onSelectSkill={onSelectSkill} />
      <IssueSection title="疑似旧版本" skills={oldVersion} onSelectSkill={onSelectSkill} />
      <IssueSection title="路径丢失" skills={missing} onSelectSkill={onSelectSkill} />
      <IssueSection title="缺少摘要 / 简介" skills={noSummary} onSelectSkill={onSelectSkill} />
    </div>
  );
}

function Card({ label, value, icon }: { label: string; value: number; icon: React.ReactNode }) {
  return (
    <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
      <div className="text-brand-600">{icon}</div>
      <p className="mt-3 text-sm text-slate-500">{label}</p>
      <p className="mt-1 text-3xl font-semibold text-slate-950">{value}</p>
    </div>
  );
}

function IssueSection({
  title,
  skills,
  onSelectSkill
}: {
  title: string;
  skills: Skill[];
  onSelectSkill: (id: number) => void;
}) {
  return (
    <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
      <h3 className="text-lg font-semibold text-slate-950">{title}</h3>
      <div className="mt-4 space-y-2">
        {skills.length === 0 ? (
          <div className="rounded-3xl border border-dashed border-slate-300 p-8 text-center text-sm text-slate-500">
            暂无问题。
          </div>
        ) : (
          skills.slice(0, 30).map((skill) => (
            <button
              key={skill.id}
              onClick={() => onSelectSkill(skill.id)}
              className="grid w-full grid-cols-[1fr_120px_120px] items-center gap-3 rounded-2xl bg-slate-50 px-4 py-3 text-left transition hover:bg-slate-100"
            >
              <span className="min-w-0">
                <span className="block truncate font-medium text-slate-800">{skill.name}</span>
                <span className="block truncate text-xs text-slate-500">{skill.path}</span>
              </span>
              <span className="text-sm text-slate-600">{skill.quality_score} 分</span>
              <span className="text-sm text-slate-600">{skill.archive_recommendation}</span>
            </button>
          ))
        )}
      </div>
    </section>
  );
}
