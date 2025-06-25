import type { RouterMiddleware } from '../types';
import { useAuthStore } from '../../stores/auth';

export const permissionsMiddleware: RouterMiddleware = async (to, from, next) => {
  const authStore = useAuthStore();
  
  // 检查角色权限
  const requiredRoles = to.meta?.roles;
  if (requiredRoles?.length) {
    const userRoles = authStore.user?.roles || [];
    const hasRole = requiredRoles.some(role => userRoles.includes(role));
    
    if (!hasRole) {
      return next('/error/403');
    }
  }

  // 检查具体权限
  const requiredPermissions = to.meta?.permissions;
  if (requiredPermissions?.length) {
    const userPermissions = authStore.user?.permissions || [];
    const hasPermission = requiredPermissions.every(permission => 
      userPermissions.includes(permission)
    );
    
    if (!hasPermission) {
      return next('/error/403');
    }
  }

  next();
};