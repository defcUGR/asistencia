const error: Ref<null | {
  msg: string;
  delay: number;
}> = ref(null);

export const useError = () => error;

let prevTimeoutId: number | null = null;

export const raiseError = (err: Error | string, delay: number = 4000) => {
  if (prevTimeoutId) clearInterval(prevTimeoutId);
  console.error(err);
  error.value = { msg: err instanceof Error ? err.message : err, delay };
  prevTimeoutId = setTimeout(() => (error.value = null), delay);
};

export const clearError = () => {
  if (prevTimeoutId) clearInterval(prevTimeoutId);
  prevTimeoutId = null;
  error.value = null;
};
