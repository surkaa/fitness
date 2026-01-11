<template>
  <q-page padding class="column">
    <div class="row items-center q-mb-md">
      <q-btn flat round icon="arrow_back" color="primary" @click="router.back()"/>
      <div class="text-h5 q-ml-sm">{{ exerciseName }}</div>
    </div>

    <q-card flat bordered class="q-mb-md bg-grey-1">
      <q-card-section>
        <div class="text-subtitle2 text-grey-7">容量走势 (Weight)</div>
      </q-card-section>

      <q-card-section class="q-pa-none" style="height: 250px;">
        <v-chart v-if="records.length > 1" class="chart" :option="chartOption" autoresize />
        <div v-else class="full-height flex flex-center text-grey-5">
          <div class="text-center">
            <q-icon name="show_chart" size="40px"/>
            <div class="text-caption q-mt-sm">至少需要两条记录才能生成趋势图</div>
          </div>
        </div>
      </q-card-section>
    </q-card>

    <div class="text-h6 q-mb-sm q-px-sm">历史记录</div>

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
            {{ record.weight }} <span class="text-caption text-grey-7">{{ unit }}</span>
          </q-item-label>
          <q-item-label caption>
            {{ formatDate(record.createdAt) }}{{ record.reps ? ` · ${record.reps} 次` : '' }}
          </q-item-label>
        </q-item-section>

        <q-item-section side>
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

  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { date, useQuasar } from 'quasar';
import { invokeStrict } from '../utils/invokeStrict';

import VChart from 'vue-echarts';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { LineChart } from 'echarts/charts';
import {
  GridComponent,
  TooltipComponent,
  TitleComponent,
  DataZoomComponent
} from 'echarts/components';
import {ExerciseRecord} from "../types.ts";
import {formatDate} from "../utils/format.ts";

use([
  CanvasRenderer,
  LineChart,
  GridComponent,
  TooltipComponent,
  TitleComponent,
  DataZoomComponent
]);

const route = useRoute();
const router = useRouter();
const $q = useQuasar();

// 获取参数
const exerciseId = Number(route.params.id);
const exerciseName = (history.state.exerciseName as string) || '动作详情';
const unit = (history.state.exerciseUnit as string) || 'kg';

const records = ref<ExerciseRecord[]>([]);
const loading = ref(false);

const chartOption = computed(() => {
  const sortedForChart = [...records.value].sort((a, b) =>
      new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
  );

  const dates = sortedForChart.map(r => date.formatDate(r.createdAt, 'MM/DD'));
  const weights = sortedForChart.map(r => r.weight);

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
      axisLine: { lineStyle: { color: '#999' } }
    },
    yAxis: {
      type: 'value',
      scale: true,
      splitLine: { lineStyle: { type: 'dashed' } }
    },
    series: [
      {
        data: weights,
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 8,
        itemStyle: { color: '#1976D2' },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(25, 118, 210, 0.3)' },
              { offset: 1, color: 'rgba(25, 118, 210, 0)' }
            ]
          }
        }
      }
    ]
  };
});

async function loadHistory() {
  try {
    records.value = await invokeStrict('page_records', {
      exerciseId,
      page: 1,
      pageSize: 1000
    }, loading);
  } catch (e) {
    $q.notify({ type: 'negative', message: '获取历史记录失败: ' + e });
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
      await invokeStrict('delete_record', { recordId });
      // 从本地列表中移除
      records.value = records.value.filter(r => r.id !== recordId);
      $q.notify({ type: 'positive', message: '已删除', timeout: 1000 });
    } catch (e) {
      $q.notify({ type: 'negative', message: String(e) });
    }
  });
}

onMounted(() => {
  loadHistory();
});
</script>