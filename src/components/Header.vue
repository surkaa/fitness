<template>
  <div class="row items-center justify-between q-mb-md">
    <div class="row items-center">
      <q-btn
          v-if="showBack"
          flat
          round
          icon="arrow_back"
          color="primary"
          @click="handleBack"
      />
      <div v-if="title" class="text-h5 q-ml-sm">{{ title }}</div>
      <slot name="left" />
    </div>
    <div>
      <slot name="right" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';

const props = defineProps<{
  title?: string;
  showBack?: boolean;
  backAction?: () => void;
}>();

const router = useRouter();
const handleBack = () => {
  if (props.backAction) {
    props.backAction();
  } else {
    router.back();
  }
};
</script>
