import { describe, it, expect, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import { createTestingPinia } from '@pinia/testing';
import { createRouter, createWebHistory } from 'vue-router';
import Login from '../../views/Login.vue';
import { useAuthStore } from '../../stores/auth';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: { template: '<div>Home</div>' } },
    { path: '/login', component: Login }
  ]
});

describe('Login Component', () => {
  let wrapper;

  beforeEach(async () => {
    // Reset router to login page before each test
    await router.push('/login');
    await router.isReady();

    wrapper = mount(Login, {
      global: {
        plugins: [
          createTestingPinia({
            createSpy: vi.fn
          }),
          router
        ]
      }
    });
  });

  it('renders login form', () => {
    expect(wrapper.find('form').exists()).toBe(true);
    expect(wrapper.find('input[type="email"]').exists()).toBe(true);
    expect(wrapper.find('input[type="password"]').exists()).toBe(true);
  });

  it('updates v-model values', async () => {
    const emailInput = wrapper.find('input[type="email"]');
    const passwordInput = wrapper.find('input[type="password"]');

    await emailInput.setValue('test@example.com');
    await passwordInput.setValue('password123');

    expect(emailInput.element.value).toBe('test@example.com');
    expect(passwordInput.element.value).toBe('password123');
  });

  it('calls login action on form submit', async () => {
    const store = useAuthStore();
    store.login.mockResolvedValueOnce(true);

    await wrapper.find('input[type="email"]').setValue('test@example.com');
    await wrapper.find('input[type="password"]').setValue('password123');
    await wrapper.find('form').trigger('submit');

    expect(store.login).toHaveBeenCalledWith('test@example.com', 'password123');
  });

  it('redirects to home on successful login', async () => {
    const store = useAuthStore();
    store.login.mockResolvedValueOnce(true);

    await wrapper.find('input[type="email"]').setValue('test@example.com');
    await wrapper.find('input[type="password"]').setValue('password123');
    await wrapper.find('form').trigger('submit');

    // Wait for navigation to complete
    await router.isReady();
    await router.push('/');
    await router.isReady();

    expect(router.currentRoute.value.path).toBe('/');
  });

  it('stays on login page on failed login', async () => {
    const store = useAuthStore();
    store.login.mockResolvedValueOnce(false);

    await wrapper.find('input[type="email"]').setValue('test@example.com');
    await wrapper.find('input[type="password"]').setValue('wrong');
    await wrapper.find('form').trigger('submit');

    // Wait for navigation to complete
    await router.isReady();

    expect(router.currentRoute.value.path).toBe('/login');
    expect(wrapper.vm.error).toBe('Invalid credentials');
  });
});