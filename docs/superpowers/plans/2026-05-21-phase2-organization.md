# Phase 2: 分类组织 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add tags, important flag, pinned, and daily recurring tasks to the TODO app.

**Architecture:** Extend the Task data model with 4 new fields (`tags: Vec<String>`, `important: bool`, `pinned: bool`, `is_daily: bool`) plus a `DailyCompletion` struct for per-date completion tracking. New `TagFilterBar` component for tag-based filtering. TaskInput gains a row of toggle buttons (⭐📌☀️🏷) for setting metadata at creation time. TaskItem displays tags, important star, and daily icon. TaskList sorts tasks (pinned → important → normal → completed). Daily tasks use a separate `toggle_daily_task(id, date)` command that inserts/removes `DailyCompletion` records.

**Tech Stack:** Tauri 2, Rust, Vue 3 + TypeScript, Vite

**Spec:** `docs/superpowers/specs/2026-05-21-phase2-organization-design.md`

---

## File map

| File | Action | Responsibility |
|------|--------|---------------|
| `src-tauri/src/store.rs` | Modify | Add fields to Task, DailyCompletion struct, TaskStore.daily_completions, update tests |
| `src-tauri/src/main.rs` | Modify | Extend add_task/update_task, add toggle_daily_task/get_all_tags/delete_tag |
| `src/types.ts` | Modify | Add tags[], important, pinned, is_daily to Task interface |
| `src/components/TagFilterBar.vue` | Create | Tag chip bar for filtering |
| `src/components/TaskInput.vue` | Modify | Toggle button row + tag input section |
| `src/components/TaskItem.vue` | Modify | Stars, daily icon, tags display, daily toggle logic |
| `src/components/TaskList.vue` | Modify | Sorting, pinned section header |
| `src/App.vue` | Modify | TagFilterBar, tag state, daily completions, updated handlers |

---

### Task 1: Extend data model (Rust + TypeScript)

**Files:**
- Modify: `src-tauri/src/store.rs`
- Modify: `src/types.ts`

- [ ] **Step 1: Add new fields to Rust Task struct**

In `src-tauri/src/store.rs`, replace the Task struct:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub due_date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub is_daily: bool,
}
```

- [ ] **Step 2: Add DailyCompletion struct and update TaskStore**

After the Task struct, add:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyCompletion {
    pub task_id: String,
    pub date: String,
}
```

Update TaskStore:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStore {
    pub version: u32,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub daily_completions: Vec<DailyCompletion>,
}
```

Update `load_tasks` default constructors to include `daily_completions: vec![]`.

Update `test_empty_store_has_no_tasks` to include `daily_completions: vec![]`.

- [ ] **Step 3: Add new fields to TypeScript Task interface**

In `src/types.ts`:

```typescript
export interface Task {
  id: string;
  title: string;
  completed: boolean;
  created_at: string;
  completed_at: string | null;
  due_date: string | null;
  tags: string[];
  important: boolean;
  pinned: boolean;
  is_daily: boolean;
}

export interface DailyCompletion {
  task_id: string;
  date: string;
}
```

- [ ] **Step 4: Run Rust tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/store.rs src/types.ts
git commit -m "feat: add tags, important, pinned, is_daily fields + DailyCompletion"
```

---

### Task 2: Update backend commands

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Extend `add_task` with new fields**

Replace the `add_task` function:

```rust
#[tauri::command]
fn add_task(
    state: tauri::State<AppState>,
    title: String,
    due_date: Option<String>,
    tags: Option<Vec<String>>,
    important: Option<bool>,
    pinned: Option<bool>,
    is_daily: Option<bool>,
) -> Result<store::Task, String> {
    let mut store = state.store.lock().unwrap();
    let task = store::Task {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        completed: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        due_date,
        tags: tags.unwrap_or_default(),
        important: important.unwrap_or(false),
        pinned: pinned.unwrap_or(false),
        is_daily: is_daily.unwrap_or(false),
    };
    store.tasks.push(task.clone());
    store::save_tasks(&store)?;
    Ok(task)
}
```

- [ ] **Step 2: Extend `update_task` with new fields**

Replace the `update_task` function:

```rust
#[tauri::command]
fn update_task(
    state: tauri::State<AppState>,
    id: String,
    title: String,
    due_date: Option<String>,
    tags: Vec<String>,
    important: bool,
    pinned: bool,
    is_daily: bool,
) -> Result<(), String> {
    let mut store = state.store.lock().unwrap();
    if let Some(task) = store.tasks.iter_mut().find(|t| t.id == id) {
        task.title = title;
        task.due_date = due_date;
        task.tags = tags;
        task.important = important;
        task.pinned = pinned;
        task.is_daily = is_daily;
    }
    store::save_tasks(&store)
}
```

- [ ] **Step 3: Add `toggle_daily_task` command**

```rust
#[tauri::command]
fn toggle_daily_task(state: tauri::State<AppState>, id: String, date: String) -> Result<(), String> {
    let mut store = state.store.lock().unwrap();
    if let Some(pos) = store.daily_completions.iter().position(|dc| dc.task_id == id && dc.date == date) {
        store.daily_completions.remove(pos);
    } else {
        store.daily_completions.push(store::DailyCompletion {
            task_id: id,
            date,
        });
    }
    store::save_tasks(&store)
}
```

- [ ] **Step 4: Add `get_all_tags` command**

```rust
#[tauri::command]
fn get_all_tags(state: tauri::State<AppState>) -> Vec<String> {
    let store = state.store.lock().unwrap();
    let mut tags: Vec<String> = store.tasks.iter()
        .flat_map(|t| t.tags.clone())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}
```

- [ ] **Step 5: Add `delete_tag` command**

```rust
#[tauri::command]
fn delete_tag(state: tauri::State<AppState>, tag: String) -> Result<(), String> {
    let mut store = state.store.lock().unwrap();
    for task in store.tasks.iter_mut() {
        task.tags.retain(|t| t != &tag);
    }
    store::save_tasks(&store)
}
```

- [ ] **Step 6: Update invoke_handler**

Add `toggle_daily_task, get_all_tags, delete_tag` to the handler list.

- [ ] **Step 7: Verify Rust compiles**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: extend backend commands for tags, daily tasks, important, pinned"
```

---

### Task 3: Create TagFilterBar component

**Files:**
- Create: `src/components/TagFilterBar.vue`

- [ ] **Step 1: Create TagFilterBar.vue**

```vue
<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  tags: string[];
  selected: string[];
}>();

const emit = defineEmits<{
  toggleTag: [tag: string];
  addTag: [tag: string];
}>();

const showInput = ref(false);
const newTagName = ref('');

function toggleTag(tag: string) {
  emit('toggleTag', tag);
}

function handleAddTag() {
  const trimmed = newTagName.value.trim();
  if (trimmed && !props.tags.includes(trimmed)) {
    emit('addTag', trimmed);
  }
  newTagName.value = '';
  showInput.value = false;
}
</script>

<template>
  <div class="tag-filter">
    <div class="tag-chips">
      <button
        :class="['tag-chip', { active: selected.length === 0 }]"
        @click="emit('toggleTag', '')"
      >
        全部
      </button>
      <button
        v-for="tag in tags"
        :key="tag"
        :class="['tag-chip', { active: selected.includes(tag) }]"
        @click="toggleTag(tag)"
      >
        {{ tag }}
      </button>
      <button
        v-if="!showInput"
        class="tag-chip add"
        @click="showInput = true"
      >
        +
      </button>
      <input
        v-else
        v-model="newTagName"
        type="text"
        class="tag-input-inline"
        placeholder="新标签"
        @keydown.enter="handleAddTag"
        @keydown.escape="showInput = false"
        @blur="handleAddTag"
      />
    </div>
  </div>
</template>

<style scoped>
.tag-filter {
  margin-bottom: 12px;
}

.tag-chips {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}

.tag-chip {
  font-size: 12px;
  padding: 4px 12px;
  border-radius: 12px;
  border: 1px solid #e0e0e0;
  background: white;
  color: #666;
  cursor: pointer;
  transition: all 0.15s;
}

.tag-chip:hover {
  border-color: #4a90d9;
  color: #4a90d9;
}

.tag-chip.active {
  background: #4a90d9;
  color: white;
  border-color: #4a90d9;
}

.tag-chip.add {
  border-style: dashed;
  color: #bbb;
}

.tag-input-inline {
  font-size: 12px;
  padding: 4px 8px;
  border: 1px solid #4a90d9;
  border-radius: 12px;
  outline: none;
  width: 80px;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/components/TagFilterBar.vue
git commit -m "feat: add TagFilterBar component"
```

---

### Task 4: Update TaskInput with new controls

**Files:**
- Modify: `src/components/TaskInput.vue`

Read the current file first, then replace entirely.

- [ ] **Step 1: Replace TaskInput.vue with enhanced version**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import DatePicker from './DatePicker.vue';

const emit = defineEmits<{
  add: [title: string, due_date: string | null, tags: string[], important: boolean, pinned: boolean, is_daily: boolean];
}>();

const title = ref('');
const showError = ref(false);
const dueDate = ref<string | null>(null);
const showPicker = ref(false);
const important = ref(false);
const pinned = ref(false);
const isDaily = ref(false);
const tags = ref<string[]>([]);
const showTagInput = ref(false);
const newTag = ref('');

function handleSubmit() {
  const trimmed = title.value.trim();
  if (!trimmed) {
    showError.value = true;
    setTimeout(() => { showError.value = false; }, 2000);
    return;
  }
  emit('add', trimmed, dueDate.value, [...tags.value], important.value, pinned.value, isDaily.value);
  title.value = '';
  dueDate.value = null;
  showPicker.value = false;
  important.value = false;
  pinned.value = false;
  isDaily.value = false;
  tags.value = [];
  showError.value = false;
}

function onDateSelect(date: string | null) {
  dueDate.value = date;
  showPicker.value = false;
}

function addTag() {
  const t = newTag.value.trim();
  if (t && !tags.value.includes(t)) {
    tags.value.push(t);
  }
  newTag.value = '';
}

function removeTag(tag: string) {
  tags.value = tags.value.filter(t => t !== tag);
}

function formatDueDate(d: string): string {
  const [y, m, day] = d.split('-');
  return `${m}/${day}`;
}
</script>

<template>
  <div class="task-input">
    <div class="input-row">
      <input
        v-model="title"
        type="text"
        placeholder="输入新任务..."
        :class="['task-input-field', { error: showError }]"
        @keydown.enter="handleSubmit"
      />
      <div class="date-btn-wrapper">
        <button
          :class="['date-btn', { active: dueDate }]"
          title="设置截止日期"
          @click="showPicker = !showPicker"
        >
          📅
        </button>
        <DatePicker
          :visible="showPicker"
          @select="onDateSelect"
        />
      </div>
      <button class="task-input-btn" @click="handleSubmit">添加</button>
    </div>

    <div class="quick-actions">
      <button
        :class="['qa-btn', { active: important }]"
        @click="important = !important"
      >
        ⭐ 重要
      </button>
      <button
        :class="['qa-btn', { active: pinned }]"
        @click="pinned = !pinned"
      >
        📌 置顶
      </button>
      <button
        :class="['qa-btn', { active: isDaily }]"
        @click="isDaily = !isDaily"
      >
        ☀️ 每日
      </button>
      <button
        :class="['qa-btn', { active: showTagInput || tags.length > 0 }]"
        @click="showTagInput = !showTagInput"
      >
        🏷 标签
      </button>
    </div>

    <div v-if="showTagInput" class="tag-input-row">
      <input
        v-model="newTag"
        type="text"
        placeholder="输入标签名..."
        class="tag-input"
        @keydown.enter.prevent="addTag"
        @keydown.escape="showTagInput = false"
      />
      <button class="tag-add-btn" @click="addTag">添加</button>
    </div>

    <div v-if="tags.length > 0" class="tag-preview">
      <span v-for="tag in tags" :key="tag" class="tag-chip">
        {{ tag }}
        <button class="tag-chip-x" @click="removeTag(tag)">×</button>
      </span>
    </div>

    <div v-if="dueDate || important || pinned || isDaily || tags.length > 0" class="summary">
      <span v-if="dueDate">📅 截止 {{ formatDueDate(dueDate) }}</span>
      <span v-if="important"> ⭐ 重要</span>
      <span v-if="pinned"> 📌 置顶</span>
      <span v-if="isDaily"> ☀️ 每日</span>
      <span v-if="tags.length > 0"> 🏷 {{ tags.join(', ') }}</span>
    </div>
  </div>
</template>

<style scoped>
.task-input { margin-bottom: 16px; }

.input-row {
  display: flex;
  gap: 8px;
}

.task-input-field {
  flex: 1;
  padding: 10px 14px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 15px;
  outline: none;
  transition: border-color 0.2s;
}

.task-input-field:focus { border-color: #4a90d9; }
.task-input-field.error { border-color: #e74c3c; }

.date-btn-wrapper { position: relative; }

.date-btn {
  padding: 10px;
  background: none;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  transition: border-color 0.2s;
}

.date-btn:hover, .date-btn.active { border-color: #4a90d9; }

.task-input-btn {
  padding: 10px 20px;
  background: #4a90d9;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  cursor: pointer;
  transition: background 0.2s;
  white-space: nowrap;
}

.task-input-btn:hover { background: #357abd; }

.quick-actions {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.qa-btn {
  padding: 4px 10px;
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
  color: #999;
}

.qa-btn:hover { border-color: #4a90d9; color: #4a90d9; }

.qa-btn.active {
  background: #e8f0fe;
  border-color: #4a90d9;
  color: #4a90d9;
}

.tag-input-row {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.tag-input {
  flex: 1;
  padding: 5px 10px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
}

.tag-input:focus { border-color: #4a90d9; }

.tag-add-btn {
  padding: 5px 12px;
  background: #4a90d9;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.tag-preview {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.tag-chip {
  font-size: 11px;
  background: #e8f0fe;
  color: #4a90d9;
  padding: 2px 8px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.tag-chip-x {
  background: none;
  border: none;
  color: #4a90d9;
  cursor: pointer;
  font-size: 14px;
  padding: 0;
  line-height: 1;
}

.summary {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
  font-size: 12px;
  color: #999;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/components/TaskInput.vue
git commit -m "feat: add important, pinned, daily, tags controls to TaskInput"
```

---

### Task 5: Update TaskItem with enhanced display + daily toggle

**Files:**
- Modify: `src/components/TaskItem.vue`

- [ ] **Step 1: Replace TaskItem.vue with enhanced version**

```vue
<script setup lang="ts">
import { ref, nextTick, computed } from 'vue';
import type { Task } from '../types';

const props = defineProps<{
  task: Task;
  isDailyCompleted: boolean;
}>();

const emit = defineEmits<{
  toggle: [id: string];
  toggleDaily: [id: string, date: string];
  update: [id: string, title: string];
  delete: [id: string];
}>();

const editing = ref(false);
const editTitle = ref('');

function startEdit() {
  editTitle.value = props.task.title;
  editing.value = true;
  nextTick(() => {
    const input = document.getElementById(`edit-${props.task.id}`) as HTMLInputElement;
    input?.focus();
    input?.select();
  });
}

function confirmEdit() {
  const trimmed = editTitle.value.trim();
  if (trimmed && trimmed !== props.task.title) {
    emit('update', props.task.id, trimmed);
  }
  editing.value = false;
}

function cancelEdit() { editing.value = false; }

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHr = Math.floor(diffMs / 3600000);
  const diffDay = Math.floor(diffMs / 86400000);
  if (diffMin < 1) return '刚刚';
  if (diffMin < 60) return `${diffMin} 分钟前`;
  if (diffHr < 24) return `${diffHr} 小时前`;
  if (diffDay < 7) return `${diffDay} 天前`;
  return date.toLocaleDateString('zh-CN');
}

const displayCompleted = computed(() => {
  if (props.task.is_daily) return props.isDailyCompleted;
  return props.task.completed;
});

function handleToggle() {
  if (props.task.is_daily) {
    const today = new Date();
    const y = today.getFullYear();
    const m = String(today.getMonth() + 1).padStart(2, '0');
    const d = String(today.getDate()).padStart(2, '0');
    emit('toggleDaily', props.task.id, `${y}-${m}-${d}`);
  } else {
    emit('toggle', props.task.id);
  }
}

const dueStatus = computed<'overdue' | 'today' | 'upcoming' | null>(() => {
  if (!props.task.due_date) return null;
  const today = new Date();
  const y = today.getFullYear();
  const m = String(today.getMonth() + 1).padStart(2, '0');
  const d = String(today.getDate()).padStart(2, '0');
  const todayStr = `${y}-${m}-${d}`;
  if (props.task.due_date < todayStr) return 'overdue';
  if (props.task.due_date === todayStr) return 'today';
  return 'upcoming';
});

const dueLabel = computed(() => {
  if (!props.task.due_date) return '';
  if (dueStatus.value === 'today') return '今天到期';
  if (dueStatus.value === 'overdue') return '已过期';
  return props.task.due_date;
});
</script>

<template>
  <div
    :class="['task-item', {
      completed: displayCompleted,
      editing: editing,
      [dueStatus || '']: !displayCompleted && dueStatus
    }]"
  >
    <input
      type="checkbox"
      class="task-checkbox"
      :checked="displayCompleted"
      @change="handleToggle"
    />

    <div class="task-body" @dblclick="startEdit">
      <template v-if="!editing">
        <div class="task-title-row">
          <span :class="['task-title', { done: displayCompleted }]">{{ task.title }}</span>
          <span v-if="task.important && !displayCompleted" class="icon-star">⭐</span>
          <span v-if="task.is_daily" class="icon-daily" :class="{ done: displayCompleted }">☀️</span>
        </div>
        <div class="task-meta">
          <span class="task-time">{{ formatTime(task.created_at) }}</span>
          <span v-if="dueLabel && !displayCompleted" :class="['due-badge', dueStatus]">{{ dueLabel }}</span>
          <span v-for="tag in task.tags" :key="tag" class="tag-badge">{{ tag }}</span>
        </div>
      </template>
      <template v-else>
        <input
          :id="`edit-${task.id}`"
          v-model="editTitle"
          type="text"
          class="task-edit-input"
          @keydown.enter="confirmEdit"
          @keydown.escape="cancelEdit"
          @blur="confirmEdit"
        />
      </template>
    </div>

    <button
      v-if="!editing"
      class="task-delete-btn"
      title="删除"
      @click="emit('delete', task.id)"
    >
      ×
    </button>
  </div>
</template>

<style scoped>
.task-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  border-left: 3px solid transparent;
  transition: background 0.15s;
}

.task-item:hover { background: #f8f9fa; }
.task-item.completed { background: #fafafa; }

.task-item.overdue { border-left-color: #e74c3c; }
.task-item.today { border-left-color: #f39c12; }
.task-item.upcoming { border-left-color: #4a90d9; }

.task-checkbox {
  width: 18px;
  height: 18px;
  margin-right: 12px;
  cursor: pointer;
  accent-color: #4a90d9;
  flex-shrink: 0;
}

.task-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  cursor: default;
}

.task-title-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.task-title {
  font-size: 15px;
  color: #333;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-title.done { text-decoration: line-through; color: #aaa; }

.icon-star { font-size: 13px; flex-shrink: 0; }
.icon-daily { font-size: 13px; flex-shrink: 0; }
.icon-daily.done { opacity: 0.4; }

.task-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 3px;
  flex-wrap: wrap;
}

.task-time {
  font-size: 12px;
  color: #999;
}

.due-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 500;
}

.due-badge.overdue { background: #fde8e8; color: #e74c3c; }
.due-badge.today { background: #fef3e0; color: #e67e22; }
.due-badge.upcoming { background: #e8f0fe; color: #4a90d9; }

.tag-badge {
  font-size: 10px;
  background: #f0f0f0;
  color: #888;
  padding: 1px 5px;
  border-radius: 3px;
}

.task-edit-input {
  font-size: 15px;
  padding: 4px 8px;
  border: 2px solid #4a90d9;
  border-radius: 6px;
  outline: none;
  width: 100%;
}

.task-delete-btn {
  background: none;
  border: none;
  color: #ccc;
  font-size: 20px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  flex-shrink: 0;
  transition: color 0.15s;
}

.task-delete-btn:hover { color: #e74c3c; }
</style>
```

Key changes:
- New prop: `isDailyCompleted: boolean` — passed from parent for daily task completion display
- New emit: `toggleDaily: [id: string, date: string]` — for daily task toggle
- `displayCompleted` computed: checks `isDailyCompleted` for daily tasks, `task.completed` for normal
- `handleToggle()`: dispatches to correct backend command based on `is_daily`
- Template: star icon, daily icon, tag badges in meta row
- `.tag-badge` styling for tags

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/components/TaskItem.vue
git commit -m "feat: add important star, daily icon, tags display + daily toggle to TaskItem"
```

---

### Task 6: Update TaskList with sorting + pinned section

**Files:**
- Modify: `src/components/TaskList.vue`

- [ ] **Step 1: Replace TaskList.vue with sorted + pinned section version**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import type { Task } from '../types';
import TaskItem from './TaskItem.vue';

const props = defineProps<{
  tasks: Task[];
  dailyCompletionsMap: Record<string, boolean>;
}>();

const emit = defineEmits<{
  toggle: [id: string];
  toggleDaily: [id: string, date: string];
  update: [id: string, title: string];
  delete: [id: string];
}>();

const sortedTasks = computed(() => {
  const arr = [...props.tasks];
  arr.sort((a, b) => {
    // 1. Pinned first
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    // 2. Important before normal
    if (a.important !== b.important) return a.important ? -1 : 1;
    // 3. Completed last
    if (a.completed !== b.completed) return a.completed ? 1 : -1;
    // 4. Newest first within same group
    return b.created_at.localeCompare(a.created_at);
  });
  return arr;
});

const pinnedTasks = computed(() => sortedTasks.value.filter(t => t.pinned && !t.completed));
const normalTasks = computed(() => sortedTasks.value.filter(t => !t.pinned || t.completed));
</script>

<template>
  <div class="task-list">
    <div v-if="tasks.length === 0" class="task-empty">
      暂无任务，添加一个吧
    </div>
    <template v-else>
      <div v-if="pinnedTasks.length > 0" class="pinned-section">
        <div class="pinned-header">📌 已置顶</div>
        <TaskItem
          v-for="task in pinnedTasks"
          :key="task.id"
          :task="task"
          :is-daily-completed="dailyCompletionsMap[task.id] ?? false"
          @toggle="(id) => emit('toggle', id)"
          @toggle-daily="(id, date) => emit('toggleDaily', id, date)"
          @update="(id, title) => emit('update', id, title)"
          @delete="(id) => emit('delete', id)"
        />
      </div>
      <div v-if="pinnedTasks.length > 0 && normalTasks.filter(t => !t.completed).length > 0" class="section-divider"></div>
      <TaskItem
        v-for="task in normalTasks"
        :key="task.id"
        :task="task"
        :is-daily-completed="dailyCompletionsMap[task.id] ?? false"
        @toggle="(id) => emit('toggle', id)"
        @toggle-daily="(id, date) => emit('toggleDaily', id, date)"
        @update="(id, title) => emit('update', id, title)"
        @delete="(id) => emit('delete', id)"
      />
    </template>
  </div>
</template>

<style scoped>
.task-list {
  background: white;
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.task-empty {
  padding: 40px 20px;
  text-align: center;
  color: #bbb;
  font-size: 15px;
}

.pinned-section {
  background: #fffdf5;
}

.pinned-header {
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #b7950b;
  background: #fef9e7;
  border-bottom: 1px solid #f0c060;
}

.section-divider {
  height: 1px;
  background: #f0f0f0;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npx vue-tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
git add src/components/TaskList.vue
git commit -m "feat: add sorting, pinned section to TaskList"
```

---

### Task 7: Integrate everything in App.vue

**Files:**
- Modify: `src/App.vue`

This is the final integration. Get daily_completions from a new backend command, build the daily completions map, add tag filtering state, add TagFilterBar, update all handlers.

- [ ] **Step 1: Add `get_daily_completions` backend command**

In `src-tauri/src/main.rs`, add:

```rust
#[tauri::command]
fn get_daily_completions(state: tauri::State<AppState>, date: String) -> Vec<String> {
    state.store.lock().unwrap()
        .daily_completions
        .iter()
        .filter(|dc| dc.date == date)
        .map(|dc| dc.task_id.clone())
        .collect()
}
```

Register it in `invoke_handler`.

- [ ] **Step 2: Replace App.vue with fully integrated version**

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Task } from './types';
import TaskInput from './components/TaskInput.vue';
import TaskList from './components/TaskList.vue';
import TaskStats from './components/TaskStats.vue';
import MiniCalendar from './components/MiniCalendar.vue';
import TagFilterBar from './components/TagFilterBar.vue';

const tasks = ref<Task[]>([]);
const filterDate = ref<string | null>(null);
const selectedTags = ref<string[]>([]);
const allTags = ref<string[]>([]);
const dailyCompletedIds = ref<string[]>([]);

onMounted(async () => {
  tasks.value = await invoke<Task[]>('get_tasks');
  allTags.value = await invoke<string[]>('get_all_tags');
  await refreshDailyCompletions();
});

function todayStr(): string {
  const today = new Date();
  const y = today.getFullYear();
  const m = String(today.getMonth() + 1).padStart(2, '0');
  const d = String(today.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

async function refreshDailyCompletions() {
  dailyCompletedIds.value = await invoke<string[]>('get_daily_completions', { date: todayStr() });
}

const dailyCompletionsMap = computed(() => {
  const map: Record<string, boolean> = {};
  for (const id of dailyCompletedIds.value) {
    map[id] = true;
  }
  return map;
});

const filteredTasks = computed(() => {
  let result = tasks.value;
  if (filterDate.value) {
    result = result.filter(t => t.due_date === filterDate.value);
  }
  if (selectedTags.value.length > 0) {
    result = result.filter(t =>
      selectedTags.value.some(tag => t.tags.includes(tag))
    );
  }
  return result;
});

const overdueCount = computed(() => {
  const ts = todayStr();
  return tasks.value.filter(t => t.due_date && t.due_date < ts && !t.completed).length;
});

async function handleAdd(
  title: string,
  due_date: string | null,
  tags: string[],
  important: boolean,
  pinned: boolean,
  is_daily: boolean,
) {
  const task = await invoke<Task>('add_task', {
    title,
    dueDate: due_date,
    tags,
    important,
    pinned,
    isDaily: is_daily,
  });
  tasks.value.push(task);
  if (tags.length > 0) {
    allTags.value = await invoke<string[]>('get_all_tags');
  }
}

async function handleToggle(id: string) {
  await invoke('toggle_task', { id });
  const task = tasks.value.find(t => t.id === id);
  if (task) {
    task.completed = !task.completed;
    task.completed_at = task.completed ? new Date().toISOString() : null;
  }
}

async function handleToggleDaily(id: string, date: string) {
  await invoke('toggle_daily_task', { id, date });
  await refreshDailyCompletions();
}

async function handleUpdate(id: string, title: string) {
  const task = tasks.value.find(t => t.id === id);
  if (!task) return;
  await invoke('update_task', {
    id,
    title,
    dueDate: task.due_date,
    tags: task.tags,
    important: task.important,
    pinned: task.pinned,
    isDaily: task.is_daily,
  });
  task.title = title;
}

async function handleDelete(id: string) {
  await invoke('delete_task', { id });
  tasks.value = tasks.value.filter(t => t.id !== id);
  allTags.value = await invoke<string[]>('get_all_tags');
}

async function handleClearCompleted() {
  await invoke('clear_completed');
  tasks.value = tasks.value.filter(t => !t.completed);
}

function handleSelectDate(date: string | null) {
  filterDate.value = date;
}

function handleToggleTag(tag: string) {
  if (!tag) {
    selectedTags.value = [];
    return;
  }
  const idx = selectedTags.value.indexOf(tag);
  if (idx >= 0) {
    selectedTags.value.splice(idx, 1);
  } else {
    selectedTags.value.push(tag);
  }
}

function handleAddTag(tag: string) {
  if (!allTags.value.includes(tag)) {
    allTags.value.push(tag);
  }
  selectedTags.value = [tag];
}
</script>

<template>
  <div class="app">
    <h1 class="app-title">TODO</h1>
    <MiniCalendar
      :tasks="tasks"
      @select-date="handleSelectDate"
    />
    <TagFilterBar
      :tags="allTags"
      :selected="selectedTags"
      @toggle-tag="handleToggleTag"
      @add-tag="handleAddTag"
    />
    <div v-if="overdueCount > 0" class="overdue-alert">
      ⚠️ {{ overdueCount }} 项任务已过期
    </div>
    <TaskInput @add="handleAdd" />
    <TaskList
      :tasks="filteredTasks"
      :daily-completions-map="dailyCompletionsMap"
      @toggle="handleToggle"
      @toggle-daily="handleToggleDaily"
      @update="handleUpdate"
      @delete="handleDelete"
    />
    <TaskStats
      :tasks="tasks"
      @clear-completed="handleClearCompleted"
    />
  </div>
</template>

<style scoped>
.app {
  max-width: 480px;
  margin: 0 auto;
  padding: 24px 20px;
  min-height: 100vh;
}

.app-title {
  font-size: 28px;
  font-weight: 700;
  color: #1a1a2e;
  margin-bottom: 16px;
  text-align: center;
}

.overdue-alert {
  background: #fde8e8;
  color: #c0392b;
  font-size: 13px;
  padding: 8px 12px;
  border-radius: 8px;
  margin-bottom: 12px;
  text-align: center;
}
</style>
```

- [ ] **Step 3: Verify TypeScript compiles + Rust builds**

```bash
npx vue-tsc --noEmit
cd src-tauri && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/App.vue src-tauri/src/main.rs
git commit -m "feat: integrate tags, daily tasks, sorting in App.vue"
```

---

### Task 8: End-to-end verification

- [ ] **Step 1: Build everything**

```bash
npx vue-tsc --noEmit
cd src-tauri && cargo build
npx vite build
```

- [ ] **Step 2: Run Rust tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 3: Manual smoke test checklist**

1. Add task with tags — tags appear as chips on the task
2. Add task with ⭐ important — star icon appears
3. Add task with 📌 pinned — appears in pinned section
4. Add task with ☀️ daily — sun icon, checkbox toggles daily_completion
5. TagFilterBar shows all existing tags, "全部" clears filter
6. Click tag → tasks filtered, click again → filter removed
7. Pinned tasks stay above normal tasks
8. Daily task checked today → shows as completed, unchecked → uncompleted
9. Title edit preserves all metadata (tags, important, pinned, daily)
10. Delete a task → tags list updated

- [ ] **Step 4: Fix any issues found, commit**

```bash
git add -A && git commit -m "fix: address issues from e2e testing"
```
