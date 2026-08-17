import { ref, onMounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

// 🛡️ 辅助嗅探：当前是否运行在真实的 Tauri 桌面物理宿主环境中
const isTauriEnv = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useTauriWindow() {
  const 是否已置顶 = ref(false)
  const 是否已最大化 = ref(false)

  const 初始化窗口状态 = async () => {
    if (!isTauriEnv()) return
    try {
      const 物理窗口 = getCurrentWebviewWindow()
      是否已置顶.value = await 物理窗口.isAlwaysOnTop()
      是否已最大化.value = await 物理窗口.isMaximized()
    } catch (e) {
      console.warn('⚠️ 获取物理窗口状态失败:', e)
    }
  }

  const 触发最小化 = async () => {
    if (!isTauriEnv()) return
    try {
      const 物理窗口 = getCurrentWebviewWindow()
      await 物理窗口.minimize()
    } catch (e) {
      console.warn('⚠️ 最小化失败:', e)
    }
  }

  const 触发最大化还原 = async () => {
    if (!isTauriEnv()) return
    try {
      const 物理窗口 = getCurrentWebviewWindow()
      const 当前最大化 = await 物理窗口.isMaximized()
      if (当前最大化) {
        await 物理窗口.unmaximize()
        是否已最大化.value = false
      } else {
        await 物理窗口.maximize()
        是否已最大化.value = true
      }
    } catch (e) {
      console.warn('⚠️ 最大化/还原失败:', e)
    }
  }

  const 触发关闭 = async () => {
    if (!isTauriEnv()) return
    try {
      const 物理窗口 = getCurrentWebviewWindow()
      await 物理窗口.close()
    } catch (e) {
      console.warn('⚠️ 关闭窗口失败:', e)
    }
  }

  const 触发置顶切换 = async () => {
    if (!isTauriEnv()) return
    try {
      const 物理窗口 = getCurrentWebviewWindow()
      const 新状态 = !是否已置顶.value
      await 物理窗口.setAlwaysOnTop(新状态)
      是否已置顶.value = 新状态
    } catch (e) {
      console.warn('⚠️ 置顶失败:', e)
    }
  }

  onMounted(() => {
    初始化窗口状态()
  })

  return {
    是否已置顶,
    是否已最大化,
    触发最小化,
    触发最大化还原,
    触发关闭,
    触发置顶切换
  }
}
