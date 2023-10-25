<template>
  <div class="fixed bottom-16 right-4 z-50">
    <Transition name="error">
      <el-tooltip
        content="Click en botón central para cerrar"
        placement="right-start"
        :effect="isDark ? 'light' : 'effect'"
      >
        <el-card
          shadow="always"
          v-if="error"
          class="relative w-72"
          @click.middle.prevent="clearError"
        >
          <div class="!flex !flex-row gap-2">
            <IconExclamationCircle
              class="text-rose-500 block h-10 w-10 flex-shrink-0"
            ></IconExclamationCircle>
            <div class="">
              <strong class="text-lg font-bold text-rose-500">Error</strong>
              <p>{{ error.msg }}</p>
            </div>
          </div>
          <el-progress
            class="absolute -bottom-5 w-[21.5rem] -left-5"
            :percentage="percentage"
            :format="() => ''"
          />
        </el-card>
      </el-tooltip>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { IconExclamationCircle } from "@tabler/icons-vue";
import { useDark, useInterval } from "@vueuse/core";
import { useError, clearError } from "../error";

const isDark = useDark();

const error = useError();

const COUNTER_INTERVAL = 50;
const { counter, pause, reset, resume } = useInterval(COUNTER_INTERVAL, {
  controls: true,
});
const percentage = ref(100);
watch(counter, (val) => {
  error.value
    ? (percentage.value =
        ((error.value.delay - val * COUNTER_INTERVAL) * 100) /
        error.value.delay)
    : {};
});

watch(error, (val) => {
  if (val === null) pause();
  else {
    reset();
    resume();
  }
});
</script>

<style>
.error-enter-from,
.error-leave-to {
  transform: translateX(180px);
  opacity: 0;
}
</style>
