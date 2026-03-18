<template>
  <q-page padding class="column no-wrap overflow-hidden" :style-fn="pageStyleFn">
    <Header :title="exerciseName" :showBack="true" :rightAction="rightAction"/>

    <q-card flat bordered class="q-mb-md bg-grey-1">
      <q-card-section>
        <div class="text-subtitle2 text-grey-7">走势</div>
      </q-card-section>

      <q-card-section class="q-pa-none" style="height: 250px;">
        <v-chart v-if="dailyAveraged.length > 1" class="chart" :option="chartOption" autoresize/>
        <div v-else class="full-height flex flex-center text-grey-5">
          <div class="text-center">
            <q-icon name="show_chart" size="40px"/>
            <div class="text-caption q-mt-sm">至少需要两条记录才能生成趋势图</div>
          </div>
        </div>
      </q-card-section>
    </q-card>

    <div class="row items-center q-mb-sm q-px-sm">
      <div class="text-h6">历史记录</div>
      <div class="text-caption text-grey q-ml-md">
        历时{{ dailyAveraged.length }}天，共训练{{ records.length }}组
      </div>
    </div>

    <div class="col scroll">
      <div v-if="records.length === 0" class="text-center text-grey q-mt-lg">
        暂无训练记录
      </div>

      <q-list separator bordered class="rounded-borders bg-white" v-else>
        <q-item v-for="record in records" :key="record.id">
          <q-item-section avatar>
            <q-avatar color="primary" text-color="white" icon="fitness_center" size="sm"/>
          </q-item-section>

          <q-item-section>
            <q-item-label class="text-weight-bold text-subtitle1">
              {{ record.weight }} <span class="text-caption text-grey-7">{{ formatUnit(unit) }}</span>
            </q-item-label>
            <q-item-label caption>
              {{ formatRecordDate(record.createdAt) }}{{ record.reps ? ` · ${record.reps} 次` : '' }}
            </q-item-label>
          </q-item-section>

          <q-item-section side style="flex-direction: row" class="items-center q-gutter-xs">
            <q-btn
                icon="edit"
                flat
                round
                color="grey-5"
                size="sm"
                @click="handleEditRecord(record)"
            />
            <q-btn
                icon="delete"
                flat
                round
                color="grey-5"
                size="sm"
                @click="handleDelete(record.id)"
            />
          </q-item-section>
        </q-item>
      </q-list>
    </div>

    <!-- 编辑记录弹窗 -->
    <q-dialog v-model="showEditDialog">
      <q-card style="min-width: 300px">
        <q-card-section>
          <div class="text-h6">编辑记录</div>
        </q-card-section>
        <q-card-section class="q-pt-none">
          <q-input
              filled
              type="number"
              v-model.number="editForm.weight"
              label="重量"
              :suffix="formatUnit(unit)"
              autofocus
          />
          <q-input
              class="q-mt-sm"
              filled
              type="number"
              v-model.number="editForm.reps"
              label="实际完成次数 (可选)"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="取消" color="primary" v-close-popup/>
          <q-btn flat label="保存" color="primary" @click="handleUpdateRecord" :loading="submitting"/>
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import {computed, onMounted, reactive, ref} from 'vue';
import {useRoute} from 'vue-router';
import {date, useQuasar} from 'quasar';
import {invokeStrict} from '../utils/invokeStrict';

import VChart from 'vue-echarts';
import {use} from 'echarts/core';
import {CanvasRenderer} from 'echarts/renderers';
import {LineChart} from 'echarts/charts';
import {DataZoomComponent, GridComponent, TitleComponent, TooltipComponent} from 'echarts/components';
import {ExerciseRecord} from "../types.ts";
import {formatNumber, formatRecordDate} from "../utils/format.ts";
import {formatUnit} from "../utils/unitConvert.ts";
import {useExerciseStore} from "../stores/exerciseStore.ts";
import {useStorage} from '@vueuse/core';
import Header from "../components/Header.vue";

use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  TitleComponent,
  DataZoomComponent
]);

const route = useRoute();
const $q = useQuasar();
const exerciseStore = useExerciseStore();

// 获取参数
const exerciseId = Number(route.params.id);
const exerciseName = (history.state.exerciseName as string) || '动作详情';
const unit = (history.state.exerciseUnit as string) || 'kg';

// 在本地持久化一个数组，记录所有“越低越好”的动作 ID
const invertedExercises = useStorage<number[]>('inverted_exercise_ids', []);

// 判断当前动作是否是越低越好
const isInverted = computed(() => invertedExercises.value.includes(exerciseId));

const records = ref<ExerciseRecord[]>([]);
const loading = ref(false);
const submitting = ref(false);

const showEditDialog = ref(false);
const editingRecord = ref<ExerciseRecord | null>(null);
const editForm = reactive({
  weight: null as number | null,
  reps: null as number | null
});

const dailyAveraged = computed(() => {
  const map = new Map<string, { total: number; count: number }>();
  for (const r of records.value) {
    const day = date.formatDate(r.createdAt, 'YYYY-MM-DD');
    const entry = map.get(day) || {total: 0, count: 0};
    entry.total += r.weight;
    entry.count += 1;
    map.set(day, entry);
  }
  // 转为数组并按日期排序
  return Array.from(map.entries())
      .map(([day, {total, count}]) => ({day, avgWeight: formatNumber(total / count)}))
      .sort((a, b) => a.day.localeCompare(b.day));
});
const chartOption = computed(() => {
  const data = dailyAveraged.value;
  const dates = data.map(d => d.day.slice(5)); // 只取月-日
  const weights = data.map(d => d.avgWeight);

  return {
    tooltip: {
      trigger: 'axis',
      formatter: '{b}: {c}'
    },
    grid: {
      left: '10%',
      right: '5%',
      top: '10%',
      bottom: '15%'
    },
    xAxis: {
      type: 'category',
      data: dates,
      boundaryGap: false,
      axisLine: {lineStyle: {color: '#999'}}
    },
    yAxis: {
      type: 'value',
      scale: true,
      inverse: isInverted.value,
      splitLine: {lineStyle: {type: 'dashed'}}
    },
    series: [
      {
        data: weights,
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 8,
        itemStyle: {color: '#1976D2'},
        areaStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              {offset: 0, color: 'rgba(25, 118, 210, 0.3)'},
              {offset: 1, color: 'rgba(25, 118, 210, 0)'}
            ]
          }
        }
      }
    ]
  };
});

const rightAction = computed(() => [{
  label: isInverted ? '设为越高越好' : '设为越低越好',
  icon: isInverted ? 'trending_up' : 'trending_down',
  action: toggleInvert
}]);

async function loadHistory() {
  try {
    records.value = await invokeStrict('get_all_records', {exerciseId}, loading);
  } catch (e) {
    $q.notify({type: 'negative', message: '获取历史记录失败: ' + e});
  }
}

function handleDelete(recordId: number) {
  $q.dialog({
    title: '删除记录',
    message: '确定要删除这条训练记录吗？',
    cancel: true,
    persistent: true
  }).onOk(async () => {
    try {
      await invokeStrict('delete_record', {recordId});
      // 从本地列表中移除
      records.value = records.value.filter(r => r.id !== recordId);
      await exerciseStore.fetchForExercise(exerciseId);  // 刷新统计
      $q.notify({type: 'positive', message: '已删除', timeout: 1000});
    } catch (e) {
      $q.notify({type: 'negative', message: String(e)});
    }
  });
}

function handleEditRecord(record: ExerciseRecord) {
  editingRecord.value = record;
  editForm.weight = record.weight;
  editForm.reps = record.reps;
  showEditDialog.value = true;
}

async function handleUpdateRecord() {
  if (!editingRecord.value || !editForm.weight) {
    $q.notify({type: 'warning', message: '请输入重量'});
    return;
  }
  try {
    await invokeStrict('update_record', {
      recordId: editingRecord.value.id,
      weight: Number(editForm.weight),
      reps: editForm.reps ? Number(editForm.reps) : null
    }, submitting);

    // 本地更新记录
    const index = records.value.findIndex(r => r.id === editingRecord.value!.id);
    if (index !== -1) {
      records.value[index] = {
        ...records.value[index],
        weight: editForm.weight,
        reps: editForm.reps
      };
    }

    // 刷新统计
    await exerciseStore.fetchForExercise(exerciseId);

    $q.notify({type: 'positive', message: '记录已更新'});
    showEditDialog.value = false;
  } catch (e) {
    $q.notify({type: 'negative', message: '更新失败: ' + e});
  }
}

function pageStyleFn(offset: number, height: number) {
  return {height: `${height - offset}px`}
}

function toggleInvert() {
  if (isInverted.value) {
    invertedExercises.value = invertedExercises.value.filter(id => id !== exerciseId);
  } else {
    invertedExercises.value.push(exerciseId);
  }
  $q.notify({
    type: 'positive',
    message: isInverted.value ? '走势图已切换为：越低越好' : '走势图已切换为：越高越好',
    position: 'top',
    timeout: 1000
  });
}

onMounted(() => {
  loadHistory();
});
</script>