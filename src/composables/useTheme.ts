import { ref } from 'vue';

export type Theme = 'light' | 'dark' | 'auto';

export function useTheme() {
  const theme = ref<Theme>('auto');

  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme;
    // Apply theme to document if needed
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark');
    } else if (newTheme === 'light') {
      document.documentElement.classList.remove('dark');
    }
  };

  return {
    theme,
    setTheme,
  };
}
