import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUIStore = defineStore('ui', () => {
  const currentView = ref<'home' | 'format' | 'pdf' | 'asr'>('home')
  const showRadar = ref(false)
  const radarMode = ref<'global' | 'local'>('global')
  const toast消息 = ref<string | null>(null)
  const toast类型 = ref<'success' | 'error' | 'info'>('success')
  const toast显示 = ref(false)
  
  // 悬浮侧边栏收起/展开状态机
  const 侧边栏收起 = ref(false)
  
  let toast定时器: number | null = null

  const 弹出提示 = (消息: string, 类型: 'success' | 'error' | 'info' = 'success') => {
    toast消息.value = 消息
    toast类型.value = 类型
    toast显示.value = true
    if (toast定时器) {
      clearTimeout(toast定时器)
      toast定时器 = null
    }
    toast定时器 = window.setTimeout(() => {
      toast显示.value = false
    }, 2500)
  }

  const 切换侧边栏 = () => {
    侧边栏收起.value = !侧边栏收起.value
  }

  return {
    currentView,
    showRadar, radarMode,
    toast消息, toast类型, toast显示, 弹出提示,
    侧边栏收起, 切换侧边栏
  }
})
