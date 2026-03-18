<template>
  <q-card class="my-card q-mb-sm" bordered flat>
    <q-item>
      <q-item-section class="cursor-pointer" @click="$emit('click')">

        <div class="row items-center q-mb-xs">
          <div class="text-weight-bold text-h6 q-mr-sm">{{ exercise.name }}</div>
          <q-badge color="grey-2" text-color="grey-8" class="q-px-sm">
            {{ formatUnit(exercise.unit) }}
          </q-badge>
        </div>

        <div class="row items-center text-caption text-grey-8 q-mb-xs q-gutter-x-md">
          <div class="row items-center">
            <q-icon name="track_changes" size="16px" class="q-mr-xs text-primary"/>
            <span>{{ exercise.targetSets }}组 × {{ exercise.targetReps }}</span>
          </div>
          <div v-if="exercise.note" class="row items-center">
            <q-icon name="speaker_notes" size="14px" class="q-mr-xs text-grey-6"/>
            <span class="text-grey-6">{{ exercise.note }}</span>
          </div>
        </div>

        <div v-if="exerciseStore.loadingStats[exercise.id]" class="text-grey-5">
          <q-spinner size="sm" /> 加载中...
        </div>

        <div
            v-if="exerciseStore.stats[exercise.id]"
            class="row items-center text-caption text-grey-7 q-gutter-x-md"
        >
          <div class="row items-center">
            <q-icon name="assessment" size="16px" class="q-mr-xs text-info"/>
            <span>{{ exerciseStore.stats[exercise.id].totalRecords }}次</span>
          </div>

          <div v-if="exerciseStore.stats[exercise.id].maxWeight !== null" class="row items-center">
            <q-icon name="fitness_center" size="16px" class="q-mr-xs text-warning"/>
            <span>{{ exerciseStore.stats[exercise.id].maxWeight }}{{ formatUnit(exercise.unit) }}</span>
          </div>

          <div v-if="exerciseStore.stats[exercise.id].lastDate" class="row items-center">
            <q-icon name="history" size="16px" class="q-mr-xs text-secondary"/>
            <span>{{ formatRecordDate(exerciseStore.stats[exercise.id].lastDate) }}</span>
          </div>
        </div>

        <div v-else class="text-caption text-grey-5 row items-center">
          <q-icon name="info" size="14px" class="q-mr-xs"/>
          <span>暂无训练记录</span>
        </div>
      </q-item-section>

      <q-item-section side>
        <div class="column items-end q-gutter-y-xs">
          <q-btn
              label="记录"
              icon="add"
              color="primary"
              unelevated
              class="q-px-sm"
              @click.stop="$emit('record', exercise)"
          />
          <div class="row q-gutter-x-xs">
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
                color="red-4"
                size="sm"
                @click.stop="$emit('delete', exercise.id)"
            />
          </div>
        </div>
      </q-item-section>
    </q-item>
  </q-card>
</template>

<script setup lang="ts">
import {formatUnit} from '../utils/unitConvert';
import {formatRecordDate} from '../utils/format';
import {useExerciseStore} from "../stores/exerciseStore.ts";
import {Exercise} from "../bindings.ts";

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
