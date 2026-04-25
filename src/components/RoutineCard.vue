<template>
  <q-card
      class="routine-card cursor-pointer q-hoverable"
      v-ripple
      @click="$emit('click')"
  >
    <q-card-section>
      <div class="row items-center no-wrap">
        <div class="col">
          <div class="row items-center q-col-gutter-sm q-mb-sm">
            <div class="col-auto">
              <q-avatar color="primary" text-color="white" icon="event_note" size="40px"/>
            </div>
            <div class="col">
              <div class="text-h6">{{ routine.name }}</div>
            </div>
          </div>

          <div v-if="routine.description" class="text-subtitle2 text-grey-7 routine-description">
            {{ routine.description }}
          </div>
        </div>

        <div class="col-auto routine-actions">
          <q-btn
              icon="edit"
              flat
              round
              color="grey-6"
              @click.stop="$emit('edit')"
          />
          <q-btn
              icon="delete"
              flat
              round
              color="grey-6"
              @click.stop="$emit('delete')"
          />
        </div>
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import {Routine} from "../bindings.ts";

defineProps<{
  routine: Routine
}>();

defineEmits<{
  (e: 'click'): void;
  (e: 'delete'): void;
  (e: 'edit'): void;
}>();
</script>

<style scoped>
.routine-card {
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.94);
  border: 1px solid rgba(27, 42, 58, 0.08);
  box-shadow: 0 10px 24px rgba(27, 42, 58, 0.06);
  transition: transform 180ms ease, box-shadow 180ms ease;
}

.routine-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 14px 32px rgba(27, 42, 58, 0.08);
}

.routine-description {
  min-height: 40px;
  line-height: 1.45;
}

.routine-actions {
  align-self: flex-start;
}
</style>
