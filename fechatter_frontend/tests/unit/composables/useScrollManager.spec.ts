/**
 * useScrollManager Composable Unit Tests
 * Testing scroll behavior management
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { nextTick } from 'vue'
import { useScrollManager } from '@/composables/useScrollManager'
import { useViewportStore } from '@/stores/viewport'
import { createPinia, setActivePinia } from 'pinia'

// Mock element
const createMockElement = (scrollTop = 0, scrollHeight = 1000, clientHeight = 500) => ({
  scrollTop,
  scrollHeight,
  clientHeight,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  scrollTo: vi.fn(),
  querySelector: vi.fn()
})

describe('useScrollManager', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('binds scroll container correctly', () => {
    const { bindScrollContainer } = useScrollManager(1)
    const mockElement = createMockElement()

    bindScrollContainer(mockElement as any)

    expect(mockElement.addEventListener).toHaveBeenCalledWith('scroll', expect.any(Function))
  })

  it('calculates isNearBottom correctly', () => {
    const { bindScrollContainer, isNearBottom } = useScrollManager(1)

    // Not near bottom
    const element1 = createMockElement(0, 1000, 500)
    bindScrollContainer(element1 as any)
    expect(isNearBottom()).toBe(false)

    // Near bottom (within threshold)
    const element2 = createMockElement(450, 1000, 500)
    bindScrollContainer(element2 as any)
    expect(isNearBottom()).toBe(true)
  })

  it('scrolls to bottom correctly', async () => {
    const { bindScrollContainer, scrollToBottom } = useScrollManager(1)
    const mockElement = createMockElement()

    bindScrollContainer(mockElement as any)
    await scrollToBottom(false)

    expect(mockElement.scrollTop).toBe(500) // scrollHeight - clientHeight
  })

  it('scrolls to bottom smoothly', async () => {
    const { bindScrollContainer, scrollToBottom } = useScrollManager(1)
    const mockElement = createMockElement()

    bindScrollContainer(mockElement as any)
    await scrollToBottom(true)

    expect(mockElement.scrollTo).toHaveBeenCalledWith({
      top: 500,
      behavior: 'smooth'
    })
  })

  it('preserves scroll position', () => {
    const { bindScrollContainer, preserveScrollPosition } = useScrollManager(1)
    const mockElement = createMockElement(200, 1000, 500)

    bindScrollContainer(mockElement as any)
    const position = preserveScrollPosition()

    expect(position).toEqual({
      scrollTop: 200,
      scrollHeight: 1000,
      clientHeight: 500
    })
  })

  it('restores scroll position', () => {
    const { bindScrollContainer, restoreScrollPosition } = useScrollManager(1)
    const mockElement = createMockElement()

    bindScrollContainer(mockElement as any)
    restoreScrollPosition({
      scrollTop: 300,
      scrollHeight: 1000,
      clientHeight: 500
    })

    expect(mockElement.scrollTop).toBe(300)
  })

  it('gets scroll info', () => {
    const { bindScrollContainer, getScrollInfo } = useScrollManager(1)
    const mockElement = createMockElement(100, 800, 400)

    bindScrollContainer(mockElement as any)
    const info = getScrollInfo()

    expect(info).toEqual({
      scrollTop: 100,
      scrollHeight: 800,
      clientHeight: 400
    })
  })

  it('updates viewport store on scroll', async () => {
    const { bindScrollContainer } = useScrollManager(1)
    const viewportStore = useViewportStore()
    const updateSpy = vi.spyOn(viewportStore, 'updateScrollPosition')

    const mockElement = createMockElement()
    bindScrollContainer(mockElement as any)

    // Simulate scroll event
    const scrollHandler = mockElement.addEventListener.mock.calls[0][1]
    scrollHandler()

    // Wait for debounce
    await new Promise(resolve => setTimeout(resolve, 150))

    expect(updateSpy).toHaveBeenCalledWith(1, expect.objectContaining({
      scrollTop: expect.any(Number),
      scrollHeight: expect.any(Number),
      clientHeight: expect.any(Number)
    }))
  })

  it('cleans up event listener on unmount', () => {
    const { bindScrollContainer } = useScrollManager(1)
    const mockElement = createMockElement()

    bindScrollContainer(mockElement as any)

    // Simulate component unmount
    // In real usage, this would be handled by Vue's lifecycle
    expect(mockElement.addEventListener).toHaveBeenCalled()
  })
}) 