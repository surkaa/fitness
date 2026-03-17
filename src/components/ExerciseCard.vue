<template>
  <q-card class="my-card q-mb-sm" bordered flat>
    <q-item>
      <q-item-section class="cursor-pointer" @click="$emit('click')">
        <q-item-label class="text-weight-bold text-h6">{{ exercise.name }}</q-item-label>
        <q-item-label caption>
          目标: {{ exercise.targetSets }}组 × {{ exercise.targetReps }}
          <span v-if="exercise.note"> | {{ exercise.note }}</span>
        </q-item-label>
        <q-item-label caption class="text-primary">
          单位: {{ formatUnit(exercise.unit) }}
        </q-item-label>

        <q-item-label caption v-if="exerciseStore.stats[exercise.id]">
          <span>共记录: {{ exerciseStore.stats[exercise.id].totalRecords }}次</span>
          <span v-if="exerciseStore.stats[exercise.id].maxWeight !== null" class="q-ml-sm">
            最大: {{ exerciseStore.stats[exercise.id].maxWeight }}{{ formatUnit(exercise.unit) }}
          </span>
          <span v-if="exerciseStore.stats[exercise.id].lastDate" class="q-ml-sm">
            上次训练时间: {{ formatRecordDate(exerciseStore.stats[exercise.id].lastDate!) }}
          </span>
        </q-item-label>
        <q-item-label caption v-else class="text-grey-5">
          暂无记录
        </q-item-label>
      </q-item-section>

      <q-item-section side>
        <div class="row q-gutter-xs">
          <q-btn
              label="记录"
              color="primary"
              unelevated
              size="sm"
              @click.stop="$emit('record', exercise)"
          />

          <q-btn
              icon="edit"
              flat
              round
              color="grey-5"
              size="sm"
              @click.stop="$emit('edit', exercise.id)"
          />

          <q-btn
              icon="delete"
              flat
              round
              color="grey-5"
              size="sm"
              @click.stop="$emit('delete', exercise.id)"
          />
        </div>
      </q-item-section>
    </q-item>
  </q-card>
</template>

<script setup lang="ts">
import type {Exercise} from '../types';
import {formatUnit} from '../utils/unitConvert';
import {formatRecordDate} from '../utils/format';
import {useExerciseStore} from "../stores/exerciseStore.ts";

defineProps<{
  exercise: Exercise;
}>();

const exerciseStore = useExerciseStore();

defineEmits<{
  (e: 'click'): void;
  (e: 'delete', id: number): void;
  (e: 'edit', id: number): void;
  (e: 'record', exercise: Exercise): void;
}>();
</script>
