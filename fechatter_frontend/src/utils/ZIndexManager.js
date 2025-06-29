/**
 * Z-Index Manager - 统一层级管理系统
 * 解决层级穿透、冲突等问题
 * 
 * Design Philosophy:
 * - Simplify: 简化层级管理，统一分配策略
 * - Anticipate: 预判冲突，智能分配层级
 * - Focus: 核心功能优先，Debug面板层级最高
 * - Perfect: 完美的层级秩序，零冲突
 */

export class ZIndexManager {
  // 层级定义 - 分层管理，避免冲突
  static layers = {
    // 基础层 (1-99)
    base: 1,
    layout: 10,
    content: 20,

    // 交互层 (100-999)
    toolbar: 100,
    hover: 200,
    focus: 300,

    // 浮层 (1000-8999)
    dropdown: 1000,
    tooltip: 1500,
    translation: 5000,      // 翻译面板高优先级确保不被遮挡
    contextMenu: 3000,      // 上下文菜单
    botPanel: 4000,         // Bot管理面板
    floating: 5000,

    // 模态层 (9000-9999)
    modalBackdrop: 9000,
    modal: 9500,
    imageViewer: 9800,

    // 系统层 (10000+)
    notification: 10000,
    toast: 10500,
    debug: 15000,           // Debug面板最高优先级
    emergency: 20000        // 紧急情况
  }

  // 分配记录 - 跟踪已分配的z-index
  static allocations = new Map()

  // 冲突检测器
  static conflicts = new Set()

  /**
   * 分配Z-Index - 智能分配策略
   * @param {string} componentId - 组件唯一标识
   * @param {string} layer - 层级类型
   * @param {number} priority - 优先级调整 (-10 到 +10)
   * @returns {number} 分配的z-index值
   */
  static allocate(componentId, layer = 'dropdown', priority = 0) {
    // 验证层级
    if (!this.layers[layer]) {
      console.warn(`[ZIndexManager] Unknown layer: ${layer}, using 'dropdown'`)
      layer = 'dropdown'
    }

    // 验证优先级范围
    priority = Math.max(-10, Math.min(10, priority))

    // 计算z-index值
    const baseZ = this.layers[layer]
    const adjustedZ = baseZ + priority

    // 检查冲突
    this.detectConflict(componentId, adjustedZ, layer)

    // 记录分配
    const allocation = {
      zIndex: adjustedZ,
      layer,
      priority,
      timestamp: Date.now(),
      active: true
    }

    this.allocations.set(componentId, allocation)

    console.log(`[ZIndexManager] Allocated z-index ${adjustedZ} to ${componentId} (${layer})`)
    return adjustedZ
  }

  /**
   * 释放Z-Index - 清理分配记录
   * @param {string} componentId - 组件标识
   */
  static release(componentId) {
    const allocation = this.allocations.get(componentId)
    if (allocation) {
      allocation.active = false
      // 延迟删除，便于调试
      setTimeout(() => {
        this.allocations.delete(componentId)
      }, 1000)

      console.log(`[ZIndexManager] Released z-index for ${componentId}`)
    }
  }

  /**
   * 冲突检测 - 检测同层级冲突
   * @param {string} componentId - 组件标识  
   * @param {number} zIndex - z-index值
   * @param {string} layer - 层级
   */
  static detectConflict(componentId, zIndex, layer) {
    const sameLayer = Array.from(this.allocations.entries())
      .filter(([id, allocation]) =>
        allocation.active &&
        allocation.layer === layer &&
        id !== componentId
      )

    if (sameLayer.length > 0) {
      console.warn(`[ZIndexManager] Potential conflict detected for ${componentId} in layer ${layer}`)

      // 记录冲突
      this.conflicts.add(`${layer}-${Date.now()}`)

      // 智能解决策略
      this.resolveConflict(componentId, layer)
    }
  }

  /**
   * 冲突解决 - 智能冲突解决策略
   * @param {string} componentId - 新组件标识
   * @param {string} layer - 层级
   */
  static resolveConflict(componentId, layer) {
    const strategy = this.getResolutionStrategy(layer)

    switch (strategy) {
      case 'close_others':
        // 关闭同层级其他组件（适用于翻译面板、上下文菜单）
        this.closeOthersInLayer(layer, componentId)
        break

      case 'stack':
        // 堆叠管理（适用于模态框）
        this.stackInLayer(layer, componentId)
        break

      case 'replace':
        // 替换策略（适用于tooltip）
        this.replaceInLayer(layer, componentId)
        break

      default:
        console.log(`[ZIndexManager] No conflict resolution for layer ${layer}`)
    }
  }

  /**
   * 获取解决策略 - 根据层级确定冲突解决策略
   * @param {string} layer - 层级
   * @returns {string} 解决策略
   */
  static getResolutionStrategy(layer) {
    const strategies = {
      translation: 'close_others',    // 翻译面板：同时只能有一个
      contextMenu: 'close_others',    // 上下文菜单：关闭其他
      botPanel: 'close_others',       // Bot面板：独占
      modal: 'stack',                 // 模态框：堆叠
      tooltip: 'replace',             // 提示：替换
      notification: 'stack'           // 通知：堆叠
    }

    return strategies[layer] || 'none'
  }

  /**
   * 关闭同层级其他组件
   * @param {string} layer - 层级
   * @param {string} exceptId - 排除的组件ID
   */
  static closeOthersInLayer(layer, exceptId) {
    const toClose = Array.from(this.allocations.entries())
      .filter(([id, allocation]) =>
        allocation.active &&
        allocation.layer === layer &&
        id !== exceptId
      )

    toClose.forEach(([id, allocation]) => {
      // 触发关闭事件
      this.triggerClose(id, allocation)
      console.log(`[ZIndexManager] Auto-closed ${id} due to conflict`)
    })
  }

  /**
   * 触发组件关闭 - 通过事件通知组件关闭
   * @param {string} componentId - 组件ID
   * @param {object} allocation - 分配信息
   */
  static triggerClose(componentId, allocation) {
    // 发送关闭事件
    window.dispatchEvent(new CustomEvent('zindex-close', {
      detail: {
        componentId,
        layer: allocation.layer,
        reason: 'conflict_resolution'
      }
    }))

    // 标记为非活跃
    allocation.active = false
  }

  /**
   * 堆叠管理 - 模态框等需要堆叠的组件
   * @param {string} layer - 层级
   * @param {string} componentId - 组件ID
   */
  static stackInLayer(layer, componentId) {
    const stackItems = Array.from(this.allocations.entries())
      .filter(([id, allocation]) =>
        allocation.active && allocation.layer === layer
      )
      .sort((a, b) => a[1].timestamp - b[1].timestamp)

    // 重新分配z-index，保持堆叠顺序
    stackItems.forEach(([id, allocation], index) => {
      const newZ = this.layers[layer] + (index * 10)
      allocation.zIndex = newZ

      console.log(`[ZIndexManager] Restacked ${id} to z-index ${newZ}`)
    })
  }

  /**
   * 替换策略 - tooltip等瞬时组件
   * @param {string} layer - 层级
   * @param {string} componentId - 组件ID
   */
  static replaceInLayer(layer, componentId) {
    // 直接关闭所有同层级组件
    this.closeOthersInLayer(layer, componentId)
  }

  /**
   * 获取层级信息 - 调试和监控用
   * @returns {object} 层级信息
   */
  static getLayerInfo() {
    const activeAllocations = Array.from(this.allocations.entries())
      .filter(([id, allocation]) => allocation.active)
      .reduce((acc, [id, allocation]) => {
        if (!acc[allocation.layer]) {
          acc[allocation.layer] = []
        }
        acc[allocation.layer].push({
          id,
          zIndex: allocation.zIndex,
          timestamp: allocation.timestamp
        })
        return acc
      }, {})

    return {
      layers: this.layers,
      allocations: activeAllocations,
      conflictCount: this.conflicts.size,
      totalActive: Array.from(this.allocations.values()).filter(a => a.active).length
    }
  }

  /**
   * 紧急重置 - 清除所有分配，恢复默认状态
   */
  static emergencyReset() {
    console.warn('[ZIndexManager] Emergency reset triggered')

    // 通知所有组件关闭
    Array.from(this.allocations.entries()).forEach(([id, allocation]) => {
      if (allocation.active) {
        this.triggerClose(id, allocation)
      }
    })

    // 清空所有记录
    this.allocations.clear()
    this.conflicts.clear()

    console.log('[ZIndexManager] Emergency reset completed')
  }

  /**
   * 调试信息 - 开发环境下的详细信息
   */
  static debugInfo() {
    if (import.meta.env.DEV) {
      const info = this.getLayerInfo()
      console.group('ZIndexManager Debug Info')
      console.log('Available Layers:', info.layers)
      console.log('Active Allocations:', info.allocations)
      console.log('Conflict Count:', info.conflictCount)
      console.log('Total Active:', info.totalActive)
      console.groupEnd()

      return info
    }
  }
}

/**
 * Vue Composable - 在Vue组件中使用Z-Index管理
 */
export function useZIndex() {
  const allocate = (componentId, layer = 'dropdown', priority = 0) => {
    return ZIndexManager.allocate(componentId, layer, priority)
  }

  const release = (componentId) => {
    ZIndexManager.release(componentId)
  }

  const getLayerInfo = () => {
    return ZIndexManager.getLayerInfo()
  }

  return {
    allocate,
    release,
    getLayerInfo,
    layers: ZIndexManager.layers
  }
}

// 🌐 全局暴露，便于调试
if (typeof window !== 'undefined') {
  window.ZIndexManager = ZIndexManager
}

export default ZIndexManager 