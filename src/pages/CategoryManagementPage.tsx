import { Plus, Save, Trash2 } from "lucide-react";
import { useState } from "react";
import { Category } from "../lib/tauri";

type Props = {
  categories: Category[];
  skillCounts: Record<number, number>;
  onCreate: (input: { name: string; nameEn?: string; color?: string }) => Promise<void>;
  onUpdate: (id: number, input: { name: string; nameEn?: string; color?: string }) => Promise<void>;
  onDelete: (id: number) => Promise<void>;
};

export function CategoryManagementPage({
  categories,
  skillCounts,
  onCreate,
  onUpdate,
  onDelete
}: Props) {
  const [newName, setNewName] = useState("");
  const [newNameEn, setNewNameEn] = useState("");
  const [newColor, setNewColor] = useState("#94a3b8");
  const [drafts, setDrafts] = useState<Record<number, Category>>({});

  function draft(category: Category) {
    return drafts[category.id] ?? category;
  }

  function updateDraft(category: Category, patch: Partial<Category>) {
    setDrafts((items) => ({
      ...items,
      [category.id]: { ...draft(category), ...patch }
    }));
  }

  async function create() {
    await onCreate({ name: newName, nameEn: newNameEn, color: newColor });
    setNewName("");
    setNewNameEn("");
    setNewColor("#94a3b8");
  }

  const popularCategories = categories
    .map((category) => ({ category, count: skillCounts[category.id] ?? 0 }))
    .filter((item) => item.count > 0)
    .sort((a, b) => b.count - a.count)
    .slice(0, 8);

  return (
    <div className="space-y-5">
      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">常用分类</h3>
        <p className="mt-2 text-sm text-slate-500">
          按当前 skill 数量排序，优先显示使用最多的分类。
        </p>
        <div className="mt-4 flex flex-wrap gap-3">
          {popularCategories.length === 0 ? (
            <span className="text-sm text-slate-500">扫描后会显示高频分类。</span>
          ) : (
            popularCategories.map(({ category, count }) => (
              <span
                key={category.id}
                className="inline-flex items-center gap-2 rounded-2xl border border-slate-200 bg-slate-50 px-4 py-2 text-sm font-medium text-slate-700"
              >
                <span className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: category.color }} />
                {category.name}
                <span className="rounded-full bg-white px-2 py-0.5 text-xs text-slate-500">{count}</span>
              </span>
            ))
          )}
        </div>
      </section>

      <section className="rounded-[2rem] border border-slate-200 bg-white p-6 shadow-sm">
        <h3 className="text-lg font-semibold text-slate-950">分类管理</h3>
        <p className="mt-2 text-sm text-slate-500">
          管理 skill 分类名称、英文名和颜色。删除分类只会让关联 skill 变为未分类，不删除文件。
        </p>
        <div className="mt-5 grid grid-cols-[220px_220px_120px_auto] gap-3">
          <input className="filter-control" value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="中文分类名" />
          <input className="filter-control" value={newNameEn} onChange={(e) => setNewNameEn(e.target.value)} placeholder="English name" />
          <input className="filter-control" value={newColor} onChange={(e) => setNewColor(e.target.value)} placeholder="#94a3b8" />
          <button className="action-button" onClick={() => void create()}>
            <Plus className="h-4 w-4" />
            新增分类
          </button>
        </div>
      </section>

      <section className="overflow-hidden rounded-[2rem] border border-slate-200 bg-white shadow-sm">
        <table className="w-full text-left text-sm">
          <thead className="bg-slate-50 text-xs text-slate-500">
            <tr>
              <th className="px-4 py-3">颜色</th>
              <th className="px-4 py-3">中文名</th>
              <th className="px-4 py-3">英文名</th>
              <th className="px-4 py-3">Skill 数</th>
              <th className="px-4 py-3 text-right">操作</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {categories.map((category) => {
              const item = draft(category);
              return (
                <tr key={category.id}>
                  <td className="px-4 py-3">
                    <input className="h-9 w-24 rounded-xl border border-slate-200 px-2" value={item.color} onChange={(e) => updateDraft(category, { color: e.target.value })} />
                  </td>
                  <td className="px-4 py-3">
                    <input className="filter-control" value={item.name} onChange={(e) => updateDraft(category, { name: e.target.value })} />
                  </td>
                  <td className="px-4 py-3">
                    <input className="filter-control" value={item.name_en} onChange={(e) => updateDraft(category, { name_en: e.target.value })} />
                  </td>
                  <td className="px-4 py-3">{skillCounts[category.id] ?? 0}</td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-2">
                      <button className="action-button" onClick={() => void onUpdate(category.id, { name: item.name, nameEn: item.name_en, color: item.color })}>
                        <Save className="h-4 w-4" />
                        保存
                      </button>
                      <button
                        className="action-button text-red-600"
                        onClick={() => {
                          if (window.confirm("删除分类只会把关联 skill 设为未分类，不会删除文件。是否继续？")) {
                            void onDelete(category.id);
                          }
                        }}
                      >
                        <Trash2 className="h-4 w-4" />
                        删除
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </section>
    </div>
  );
}
