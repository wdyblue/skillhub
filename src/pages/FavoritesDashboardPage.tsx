import { BarChart3, Clock, Flame, Moon, Star } from "lucide-react";
import { Skill } from "../lib/tauri";

type Props = {
  skills: Skill[];
  onSelectSkill: (id: number) => void;
};

const DAY_MS = 24 * 60 * 60 * 1000;

export function FavoritesDashboardPage({ skills, onSelectSkill }: Props) {
  const now = Date.now();
  const totalCalls = skills.reduce((sum, skill) => sum + skill.usage_count, 0);
  const calledSkills = skills.filter((skill) => skill.usage_count > 0);
  const neverCalled = skills.filter((skill) => skill.usage_count === 0);
  const notCalled30 = skills.filter((skill) => daysSinceLastUse(skill, now) >= 30);
  const notCalled90 = skills.filter((skill) => daysSinceLastUse(skill, now) >= 90);
  const active30 = skills.filter((skill) => {
    const days = daysSinceLastUse(skill, now);
    return skill.usage_count > 0 && days < 30;
  });
  const topSkills = [...skills]
    .filter((skill) => skill.usage_count > 0)
    .sort((a, b) => b.usage_count - a.usage_count)
    .slice(0, 20);

  return (
    <div className="space-y-5">
      <section className="grid grid-cols-5 gap-4">
        <Metric label="总调用次数" value={totalCalls} icon={<BarChart3 className="h-5 w-5" />} />
        <Metric label="已调用技能" value={calledSkills.length} icon={<Star className="h-5 w-5" />} />
        <Metric label="30 天活跃" value={active30.length} icon={<Flame className="h-5 w-5" />} />
        <Metric label="30 天未调用" value={notCalled30.length} icon={<Clock className="h-5 w-5" />} />
        <Metric label="从未调用" value={neverCalled.length} icon={<Moon className="h-5 w-5" />} />
      </section>

      <section className="grid grid-cols-2 gap-5">
        <Board title="调用率最高的技能" skills={topSkills} onSelectSkill={onSelectSkill} mode="usage" />
        <Board title="30 天未调用" skills={notCalled30.slice(0, 20)} onSelectSkill={onSelectSkill} mode="stale" />
        <Board title="90 天未调用" skills={notCalled90.slice(0, 20)} onSelectSkill={onSelectSkill} mode="stale" />
        <Board title="从未调用" skills={neverCalled.slice(0, 20)} onSelectSkill={onSelectSkill} mode="usage" />
      </section>
    </div>
  );
}

function Metric({ label, value, icon }: { label: string; value: number; icon: React.ReactNode }) {
  return (
    <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
      <div className="text-brand-600">{icon}</div>
      <p className="mt-3 text-sm text-slate-500">{label}</p>
      <p className="mt-1 text-3xl font-semibold text-slate-950">{value}</p>
    </div>
  );
}

function Board({
  title,
  skills,
  onSelectSkill,
  mode
}: {
  title: string;
  skills: Skill[];
  onSelectSkill: (id: number) => void;
  mode: "usage" | "stale";
}) {
  return (
    <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
      <h3 className="text-lg font-semibold text-slate-950">{title}</h3>
      <div className="mt-4 space-y-2">
        {skills.length === 0 ? (
          <div className="rounded-3xl border border-dashed border-slate-300 p-8 text-center text-sm text-slate-500">
            暂无数据。
          </div>
        ) : (
          skills.map((skill, index) => (
            <button
              key={skill.id}
              onClick={() => onSelectSkill(skill.id)}
              className="grid w-full grid-cols-[36px_1fr_110px] items-center gap-3 rounded-2xl bg-slate-50 px-4 py-3 text-left transition hover:bg-slate-100"
            >
              <span className="text-sm font-semibold text-slate-400">#{index + 1}</span>
              <span className="min-w-0">
                <span className="block truncate font-medium text-slate-800">{skill.name}</span>
                <span className="block truncate text-xs text-slate-500">{skill.category_name ?? "未分类"}</span>
              </span>
              <span className="text-right text-sm text-slate-600">
                {mode === "usage" ? `${skill.usage_count} 次` : lastUseLabel(skill)}
              </span>
            </button>
          ))
        )}
      </div>
    </section>
  );
}

function daysSinceLastUse(skill: Skill, now: number) {
  if (!skill.last_used_at) return Number.POSITIVE_INFINITY;
  const parsed = Date.parse(skill.last_used_at.replace(" ", "T"));
  if (Number.isNaN(parsed)) return Number.POSITIVE_INFINITY;
  return Math.floor((now - parsed) / DAY_MS);
}

function lastUseLabel(skill: Skill) {
  if (!skill.last_used_at) return "从未调用";
  const days = daysSinceLastUse(skill, Date.now());
  if (!Number.isFinite(days)) return "从未调用";
  if (days <= 0) return "今天";
  return `${days} 天前`;
}
