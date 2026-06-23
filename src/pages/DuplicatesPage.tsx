import { AlertTriangle, Copy, ShieldCheck } from "lucide-react";
import { Skill } from "../lib/tauri";

type Props = {
  skills: Skill[];
  onSelectSkill: (id: number) => void;
};

export function DuplicatesPage({ skills, onSelectSkill }: Props) {
  const hashGroups = Object.values(
    skills.reduce<Record<string, Skill[]>>((acc, skill) => {
      if (!skill.hash) return acc;
      acc[skill.hash] = [...(acc[skill.hash] ?? []), skill];
      return acc;
    }, {})
  ).filter((group) => group.length > 1);

  const riskSkills = skills
    .filter((skill) => skill.duplicate_score > 0 || /copy|backup|final|v\d+|旧版|副本/i.test(skill.name))
    .sort((a, b) => b.duplicate_score - a.duplicate_score);

  return (
    <div className="space-y-5">
      <section className="grid grid-cols-3 gap-4">
        <StatCard label="完全重复组" value={hashGroups.length} icon={<Copy className="h-5 w-5" />} />
        <StatCard label="疑似重复 Skill" value={riskSkills.length} icon={<AlertTriangle className="h-5 w-5" />} />
        <StatCard label="建议处理" value={riskSkills.filter((s) => s.duplicate_score >= 35).length} icon={<ShieldCheck className="h-5 w-5" />} />
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">完全重复组</h3>
        <p className="mt-2 text-sm text-slate-500">hash 相同的 skill 内容完全一致，建议保留一个，其余归档。</p>
        <div className="mt-4 space-y-3">
          {hashGroups.length === 0 ? (
            <Empty text="未发现 hash 完全重复的 skill。" />
          ) : (
            hashGroups.map((group) => (
              <div key={group[0].hash} className="rounded-3xl border border-slate-200 p-4">
                <p className="text-sm font-semibold text-slate-800">重复组：{group.length} 个</p>
                <div className="mt-3 grid gap-2">
                  {group.map((skill) => (
                    <SkillRow key={skill.id} skill={skill} onSelectSkill={onSelectSkill} />
                  ))}
                </div>
              </div>
            ))
          )}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">疑似重复 / 旧版本</h3>
        <div className="mt-4 space-y-2">
          {riskSkills.length === 0 ? (
            <Empty text="未发现疑似重复或旧版本命名。" />
          ) : (
            riskSkills.map((skill) => <SkillRow key={skill.id} skill={skill} onSelectSkill={onSelectSkill} />)
          )}
        </div>
      </section>
    </div>
  );
}

function StatCard({ label, value, icon }: { label: string; value: number; icon: React.ReactNode }) {
  return (
    <div className="rounded-[2rem] border border-slate-200 bg-white p-5 shadow-sm">
      <div className="text-brand-600">{icon}</div>
      <p className="mt-3 text-sm text-slate-500">{label}</p>
      <p className="mt-1 text-3xl font-semibold text-slate-950">{value}</p>
    </div>
  );
}

function SkillRow({ skill, onSelectSkill }: { skill: Skill; onSelectSkill: (id: number) => void }) {
  return (
    <button
      onClick={() => onSelectSkill(skill.id)}
      className="flex items-center justify-between rounded-2xl bg-slate-50 px-4 py-3 text-left transition hover:bg-slate-100"
    >
      <span className="min-w-0">
        <span className="block truncate font-medium text-slate-800">{skill.name}</span>
        <span className="block truncate text-xs text-slate-500">{skill.path}</span>
      </span>
      <span className="ml-3 rounded-full bg-red-50 px-2.5 py-1 text-xs font-semibold text-red-600">
        风险 {skill.duplicate_score}
      </span>
    </button>
  );
}

function Empty({ text }: { text: string }) {
  return <div className="rounded-3xl border border-dashed border-slate-300 p-8 text-center text-sm text-slate-500">{text}</div>;
}
