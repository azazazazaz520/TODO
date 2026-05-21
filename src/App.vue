<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Task } from './types';
import TaskInput from './components/TaskInput.vue';
import TaskList from './components/TaskList.vue';
import TaskStats from './components/TaskStats.vue';
import MiniCalendar from './components/MiniCalendar.vue';

const tasks = ref<Task[]>([]);
const filterDate = ref<string | null>(null);

onMounted(async () => {
  tasks.value = await invoke<Task[]>('get_tasks');
});

const filteredTasks = computed(() => {
  if (!filterDate.value) return tasks.value;
  return tasks.value.filter(t => t.due_date === filterDate.value);
});

const overdueCount = computed(() => {
  const today = new Date();
  const y = today.getFullYear();
  const m = String(today.getMonth() + 1).padStart(2, '0');
  const d = String(today.getDate()).padStart(2, '0');
  const todayStr = `${y}-${m}-${d}`;
  return tasks.value.filter(t => t.due_date && t.due_date < todayStr && !t.completed).length;
});

async function handleAdd(title: string, due_date: string | null) {
  const task = await invoke<Task>('add_task', { title, dueDate: due_date });
  tasks.value.push(task);
}

async function handleToggle(id: string) {
  await invoke('toggle_task', { id });
  const task = tasks.value.find(t => t.id === id);
  if (task) {
    task.completed = !task.completed;
    task.completed_at = task.completed ? new Date().toISOString() : null;
  }
}

async function handleUpdate(id: string, title: string) {
  const task = tasks.value.find(t => t.id === id);
  const dueDate = task?.due_date ?? null;
  await invoke('update_task', { id, title, dueDate });
  if (task) task.title = title;
}

async function handleDelete(id: string) {
  await invoke('delete_task', { id });
  tasks.value = tasks.value.filter(t => t.id !== id);
}

async function handleClearCompleted() {
  await invoke('clear_completed');
  tasks.value = tasks.value.filter(t => !t.completed);
}

function handleSelectDate(date: string | null) {
  filterDate.value = date;
}
</script>

<template>
  <div class="app">
    <h1 class="app-title">TODO</h1>
    <MiniCalendar
      :tasks="tasks"
      @select-date="handleSelectDate"
    />
    <div v-if="overdueCount > 0" class="overdue-alert">
      ⚠️ {{ overdueCount }} 项任务已过期
    </div>
    <TaskInput @add="handleAdd" />
    <TaskList
      :tasks="filteredTasks"
      @toggle="handleToggle"
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
