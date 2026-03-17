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

        <q-item-label caption v-if="stats">
          <span>总记录: {{ stats.totalRecords }}次</span>
          <span v-if="stats.maxWeight !== null" class="q-ml-sm">
            最大: {{ stats.maxWeight }}{{ formatUnit(exercise.unit) }}
          </span>
          <span v-if="stats.lastDate" class="q-ml-sm">
            上次训练时间: {{ formatRecordDate(stats.lastDate) }}
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
import type {Exercise, ExerciseStats} from '../types';
import {formatUnit} from '../utils/unitConvert';
import {formatRecordDate} from '../utils/format';

defineProps<{
  exercise: Exercise;
  stats?: ExerciseStats;
}>();

defineEmits<{
  (e: 'click'): void;
  (e: 'delete', id: number): void;
  (e: 'edit', id: number): void;
  (e: 'record', exercise: Exercise): void;
}>();
</script>
