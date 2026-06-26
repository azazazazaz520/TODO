<script setup lang="ts">
import type { AppModule, ModuleDescriptor } from '../types';

const props = defineProps<{
  /** 当前激活的功能模块 */
  activeModule: AppModule;
  /** 侧边栏顶部模块列表 */
  topModules: ModuleDescriptor[];
  /** 侧边栏底部模块列表 */
  bottomModules: ModuleDescriptor[];
  /** 动作模块（悬浮窗） */
  actionModules: ModuleDescriptor[];
}>();

const emit = defineEmits<{
  switchModule: [module: AppModule];
}>();

function handleClick(item: ModuleDescriptor) {
  emit('switchModule', item.id);
}
</script>

<template>
  <nav class="sidebar">
    <!-- 顶部导航区：视图模块 -->
    <div class="sidebar-group">
      <div
        v-for="item in topModules"
        :key="item.id"
        :class="['nav-item', { active: activeModule === item.id }]"
        @click="handleClick(item)"
      >
        <svg
          class="nav-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path :d="item.iconPath" />
        </svg>
        <span>{{ item.label }}</span>
      </div>
    </div>

    <!-- 底部导航区：动作模块 + 设置 -->
    <div class="sidebar-group sidebar-bottom">
      <div v-for="item in actionModules" :key="item.id" class="nav-item" @click="handleClick(item)">
        <svg
          class="nav-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path :d="item.iconPath" />
        </svg>
        <span>{{ item.label }}</span>
      </div>
      <div
        v-for="item in bottomModules"
        :key="item.id"
        :class="['nav-item', { active: activeModule === item.id }]"
        @click="handleClick(item)"
      >
        <svg
          class="nav-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path :d="item.iconPath" />
        </svg>
        <span>{{ item.label }}</span>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.sidebar {
  width: 240px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  padding: var(--space-lg) var(--space-md);
  flex-shrink: 0;
  user-select: none;
}

.sidebar-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.sidebar-bottom {
  margin-top: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--text-secondary);
  transition: all var(--transition-fast);
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--bg-primary);
  color: var(--accent);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.nav-icon {
  width: 20px;
  height: 20px;
  stroke-width: 1.5;
}
</style>
