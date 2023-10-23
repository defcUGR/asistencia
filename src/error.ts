const error: Ref<null | {
  msg: string;
  delay: number;
}> = ref(null);

export const useError = () => error;

export const raiseError = (err: Error | string, delay: number = 4000) => {
  error.value = { msg: err instanceof Error ? err.message : err, delay };
  setTimeout(() => (error.value = null), delay);
};
