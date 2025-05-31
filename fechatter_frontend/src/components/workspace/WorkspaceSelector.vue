<template>
  <div class="relative">
    <!-- Workspace Selector Button -->
    <button
      @click="toggleDropdown"
      :disabled="loading"
      class="w-full flex items-center justify-between px-3 py-2 text-sm bg-purple-800 hover:bg-purple-700 text-white rounded-md transition-colors"
      :class="{ 'opacity-50 cursor-not-allowed': loading }"
    >
      <div class="flex items-center space-x-2 min-w-0">
        <div class="w-3 h-3 bg-green-400 rounded-full flex-shrink-0"></div>
        <span class="truncate font-medium">{{ currentWorkspace?.name || 'Select Workspace' }}</span>
      </div>
      <svg
        class="w-4 h-4 transition-transform"
        :class="{ 'rotate-180': isOpen }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
      </svg>
    </button>

    <!-- Dropdown Menu -->
    <transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="isOpen"
        class="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-md shadow-lg z-50 max-h-64 overflow-y-auto"
      >
        <!-- Loading State -->
        <div v-if="loadingWorkspaces" class="p-3 text-center">
          <div class="animate-spin w-4 h-4 border-2 border-purple-600 border-t-transparent rounded-full mx-auto"></div>
          <p class="text-xs text-gray-500 mt-2">Loading workspaces...</p>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="p-3 text-center">
          <p class="text-xs text-red-600">{{ error }}</p>
          <button
            @click="fetchWorkspaces"
            class="text-xs text-purple-600 hover:text-purple-700 mt-1"
          >
            Retry
          </button>
        </div>

        <!-- Workspace List -->
        <div v-else-if="workspaces.length > 0" class="py-1">
          <button
            v-for="workspace in workspaces"
            :key="workspace.id"
            @click="selectWorkspace(workspace)"
            :disabled="switching || workspace.id === currentWorkspace?.id"
            class="w-full px-3 py-2 text-left hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            :class="{
              'bg-purple-50 text-purple-700': workspace.id === currentWorkspace?.id,
              'text-gray-900': workspace.id !== currentWorkspace?.id
            }"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-2 min-w-0">
                <div
                  class="w-2 h-2 rounded-full flex-shrink-0"
                  :class="workspace.id === currentWorkspace?.id ? 'bg-purple-500' : 'bg-gray-300'"
                ></div>
                <span class="text-sm truncate">{{ workspace.name }}</span>
              </div>
              <div v-if="workspace.id === currentWorkspace?.id" class="text-purple-500">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                </svg>
              </div>
              <div v-else-if="switching && switchingToId === workspace.id" class="text-purple-500">
                <div class="animate-spin w-3 h-3 border border-purple-500 border-t-transparent rounded-full"></div>
              </div>
            </div>
          </button>
        </div>

        <!-- Empty State -->
        <div v-else class="p-3 text-center">
          <p class="text-xs text-gray-500">No workspaces available</p>
        </div>

        <!-- Refresh Button -->
        <div class="border-t border-gray-100 p-2">
          <button
            @click="fetchWorkspaces"
            :disabled="loadingWorkspaces"
            class="w-full px-2 py-1 text-xs text-gray-600 hover:text-gray-800 hover:bg-gray-50 rounded"
          >
            <span class="flex items-center justify-center space-x-1">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
              </svg>
              <span>Refresh</span>
            </span>
          </button>
        </div>
      </div>
    </transition>

    <!-- Click Outside Overlay -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-40"
      @click="closeDropdown"
    ></div>
  </div>
</template>

<script>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useWorkspaceStore } from '@/stores/workspace'
import { useNotifications } from '@/composables/useNotifications'
import api from '@/services/api'

export default {
  name: 'WorkspaceSelector',
  setup() {
    const authStore = useAuthStore()
    const workspaceStore = useWorkspaceStore()
    const { notifySuccess, notifyError } = useNotifications()

    // Reactive state
    const isOpen = ref(false)
    const loading = ref(false)
    const loadingWorkspaces = ref(false)
    const switching = ref(false)
    const switchingToId = ref(null)
    const workspaces = ref([])
    const error = ref(null)

    // Computed
    const currentWorkspace = computed(() => ({
      id: authStore.user?.workspace_id,
      name: workspaceStore.workspaceName
    }))

    // Methods
    const toggleDropdown = async () => {
      if (!isOpen.value && workspaces.value.length === 0) {
        await fetchWorkspaces()
      }
      isOpen.value = !isOpen.value
    }

    const closeDropdown = () => {
      isOpen.value = false
    }

    const fetchWorkspaces = async () => {
      loadingWorkspaces.value = true
      error.value = null
      
      try {
        const response = await api.get('/workspaces')
        workspaces.value = response.data || []
      } catch (err) {
        console.error('Failed to fetch workspaces:', err)
        error.value = 'Failed to load workspaces'
        notifyError('Failed to load workspaces')
      } finally {
        loadingWorkspaces.value = false
      }
    }

    const selectWorkspace = async (workspace) => {
      if (switching.value || workspace.id === currentWorkspace.value?.id) {
        return
      }

      switching.value = true
      switchingToId.value = workspace.id
      error.value = null

      try {
        const response = await api.post('/user/switch-workspace', {
          workspace_id: workspace.id
        })

        if (response.data.success) {
          // Update stores
          authStore.user.workspace_id = workspace.id
          workspaceStore.setWorkspaceName(workspace.name)
          
          notifySuccess(`Switched to workspace: ${workspace.name}`)
          closeDropdown()
          
          // Optionally reload the page to refresh all data
          setTimeout(() => {
            window.location.reload()
          }, 1000)
        } else {
          throw new Error(response.data.message || 'Failed to switch workspace')
        }
      } catch (err) {
        console.error('Failed to switch workspace:', err)
        const message = err.response?.data?.message || err.message || 'Failed to switch workspace'
        error.value = message
        notifyError(message)
      } finally {
        switching.value = false
        switchingToId.value = null
      }
    }

    // Lifecycle
    onMounted(() => {
      // Auto-fetch workspaces when component mounts
      fetchWorkspaces()
    })

    // Close dropdown when clicking outside
    const handleClickOutside = (event) => {
      if (isOpen.value && !event.target.closest('.workspace-selector')) {
        closeDropdown()
      }
    }

    onMounted(() => {
      document.addEventListener('click', handleClickOutside)
    })

    onBeforeUnmount(() => {
      document.removeEventListener('click', handleClickOutside)
    })

    return {
      // State
      isOpen,
      loading,
      loadingWorkspaces,
      switching,
      switchingToId,
      workspaces,
      error,
      // Computed
      currentWorkspace,
      // Methods
      toggleDropdown,
      closeDropdown,
      fetchWorkspaces,
      selectWorkspace,
    }
  }
}
</script>

<style scoped>
.workspace-selector {
  /* Custom styles if needed */
}
</style> 