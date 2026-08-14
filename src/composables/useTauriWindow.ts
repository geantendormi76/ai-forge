import { ref, onMounted } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

export function useTauriWindow() {
  const 是否已置顶 = ref(false)
  const 是否已最大化 = ref(false)
  const 物理窗口 = getCurrentWebviewWindow()

  const 初始化窗口状态 = async () => {
    try {
      是否已置顶.value = await 物理窗口.isAlwaysOnTop()
      是否已最大化.value = await 物理窗口.isMaximized()
    } catch (e) {
      console.warn('⚠️ 获取物理窗口状态失败:', e)
    }
  }

  const 触发最小化 = async () => {
    await 物理窗口.minimize()
  }

  const 触发最大化还原 = async () => {
    const 当前最大化 = await 物理窗口.isMaximized()
    if (当前最大化) {
      await 物理窗口.unmaximize()
      是否已最大化.value = false
    } else {
      await 物理窗口.maximize()
      是否已最大化.value = true
    }
  }

  const 触发关闭 = async () => {
    await 物理窗口.close()
  }

  const 触发置顶切换 = async () => {
    const 新状态 = !是否已置顶.value
    await 物理窗口.setAlwaysOnTop(新状态)
    是否已置顶.value = 新状态
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
