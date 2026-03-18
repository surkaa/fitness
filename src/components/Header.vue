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
      <slot name="left"/>
    </div>
    <div>
      <q-btn flat round icon="more_vert" v-if="rightAction && rightAction.length">
        <q-menu>
          <q-list style="min-width: 150px">
            <q-item v-for="action in rightAction" clickable v-close-popup @click="action.action">
              <q-item-section avatar>
                <q-icon :name="action.icon"/>
              </q-item-section>
              <q-item-section>{{ action.label }}</q-item-section>
            </q-item>
          </q-list>
        </q-menu>
      </q-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import {useRouter} from 'vue-router';
import {HeaderRightAction} from "../types.ts";

const props = defineProps<{
  title?: string;
  showBack?: boolean;
  backAction?: () => void;
  rightAction?: HeaderRightAction[]
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
