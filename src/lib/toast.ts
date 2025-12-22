import { ref } from "vue";

export type ToastType = "success" | "error" | "warning" | "info";

export interface ToastOptions {
  message: string;
  type?: ToastType;
  duration?: number;
}

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  timeout: NodeJS.Timeout;
}

const toasts = ref<Toast[]>([]);

export function useToast() {
  const createToast = ({
    message,
    type = "info",
    duration = 3000
  }: ToastOptions) => {
    if (duration < 0) {
      return;
    }

    const id = Date.now();
    const timeout = setTimeout(() => destroyToast(id), duration);

    toasts.value.push({
      id,
      message,
      type,
      timeout
    });
  };

  const destroyToast = (id: number) => {
    const idx = toasts.value.findIndex(t => t.id === id);
    toasts.value[idx].timeout.close();
    toasts.value.splice(idx, 1);
  };

  return {
    toasts,
    createToast,
    destroyToast
  };
}
