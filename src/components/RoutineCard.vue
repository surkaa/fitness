<template>
  <q-card
      class="cursor-pointer q-hoverable bg-grey-2"
      v-ripple
      @click="$emit('click')"
  >
    <q-card-section>
      <div class="row items-center no-wrap">
        <div class="col">
          <div class="text-h6">{{ routine.name }}</div>
          <div class="text-subtitle2 text-grey-7">
            {{ routine.description || '无描述' }}
          </div>
        </div>

        <div class="col-auto">
          <q-btn
              icon="delete"
              flat
              round
              color="grey-6"
              @click.stop="$emit('delete', routine.id)"
          />
        </div>
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import {Routine} from "../types";

defineProps<{
  routine: Routine
}>();

defineEmits<{
  (e: 'click'): void;
  (e: 'delete', id: number): void;
}>();
</script>
