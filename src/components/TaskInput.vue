<script setup lang="ts">
import { ref } from 'vue';
import DatePicker from './DatePicker.vue';

const emit = defineEmits<{
  add: [title: string, due_date: string | null];
}>();

const title = ref('');
const showError = ref(false);
const dueDate = ref<string | null>(null);
const showPicker = ref(false);

function handleSubmit() {
  const trimmed = title.value.trim();
  if (!trimmed) {
    showError.value = true;
    setTimeout(() => { showError.value = false; }, 2000);
    return;
  }
  emit('add', trimmed, dueDate.value);
  title.value = '';
  dueDate.value = null;
  showError.value = false;
  showPicker.value = false;
}

function onDateSelect(date: string | null) {
  dueDate.value = date;
  showPicker.value = false;
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
    <div v-if="dueDate" class="due-preview">
      截止: {{ formatDueDate(dueDate) }}
      <button class="due-preview-clear" @click="dueDate = null">×</button>
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

.date-btn-wrapper {
  position: relative;
}

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

.due-preview {
  margin-top: 6px;
  font-size: 13px;
  color: #4a90d9;
  display: flex;
  align-items: center;
  gap: 6px;
}

.due-preview-clear {
  background: none;
  border: none;
  color: #ccc;
  cursor: pointer;
  font-size: 16px;
  padding: 0;
  line-height: 1;
}

.due-preview-clear:hover { color: #e74c3c; }
</style>
