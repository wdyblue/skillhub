import { FilePlus2 } from "lucide-react";
import { Skill } from "../lib/tauri";

type Props = {
  skills: Skill[];
  onSelectSkill: (id: number) => void;
  onGoRepository: () => void;
};

export function CustomSkillsPage({ skills, onSelectSkill, onGoRepository }: Props) {
  const customSkills = skills.filter(
    (skill) => skill.is_custom || skill.source === "自建" || skill.platform === "主仓库"
  );

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-slate-950">自建技能</h3>
            <p className="mt-2 text-sm text-slate-500">
              汇总你自己创建、导入到主仓库或手动标记为自建的 skill。
            </p>
          </div>
          <button className="action-button" onClick={onGoRepository}>
            <FilePlus2 className="h-4 w-4" />
            新建 / 导入
          </button>
        </div>
      </section>

      <section className="grid grid-cols-3 gap-4">
        {customSkills.length === 0 ? (
          <div className="col-span-3 rounded-[2rem] border border-dashed border-slate-300 bg-white px-6 py-16 text-center text-sm text-slate-500">
            暂无自建技能。可以到“统一仓库”页新建或导入。
          </div>
        ) : (
          customSkills.map((skill) => (
            <button
              key={skill.id}
              onClick={() => onSelectSkill(skill.id)}
              className="rounded-[2rem] border border-slate-200 bg-white p-5 text-left shadow-sm transition hover:-translate-y-0.5 hover:shadow-soft"
            >
              <h3 className="line-clamp-2 text-lg font-semibold text-slate-950">{skill.name}</h3>
              <p className="mt-2 line-clamp-3 text-sm leading-6 text-slate-600">
                {skill.description || "暂无描述。"}
              </p>
              <div className="mt-4 flex flex-wrap gap-2 text-xs">
                <span className="rounded-full bg-slate-100 px-2.5 py-1 text-slate-600">{skill.category_name ?? "未分类"}</span>
                <span className="rounded-full bg-slate-100 px-2.5 py-1 text-slate-600">{skill.status}</span>
                <span className="rounded-full bg-brand-50 px-2.5 py-1 text-brand-700">{skill.archive_recommendation}</span>
              </div>
              <p className="mt-4 truncate text-xs text-slate-500">{skill.path}</p>
            </button>
          ))
        )}
      </section>
    </div>
  );
}
