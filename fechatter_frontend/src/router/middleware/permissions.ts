import type { RouterMiddleware } from '../types';
import { useAuthStore } from '../../stores/auth';

export const permissionsMiddleware: RouterMiddleware = async (to, _from, next) => {
  const authStore = useAuthStore();

  const requiredRoles = to.meta?.roles as string[] | undefined;
  if (requiredRoles?.length) {
    const userRoles: string[] = authStore.user?.roles || [];
    const hasRole = requiredRoles.some((role: string) => userRoles.includes(role));
    if (!hasRole) {
      next('/unauthorized');
      return;
    }
  }

  // Check permissions
  const requiredPermissions = to.meta?.permissions as string[] | undefined;
  if (requiredPermissions?.length) {
    const userPermissions: string[] = authStore.user?.permissions || [];
    const hasPermission = requiredPermissions.every((permission: string) =>
      userPermissions.includes(permission)
    );
    if (!hasPermission) {
      next('/forbidden');
      return;
    }
  }

  next();
};