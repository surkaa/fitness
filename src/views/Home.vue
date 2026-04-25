<template>
  <q-page class="calendar-page q-pa-md column no-wrap">
    <Header
        :title="monthTitle(currentMonth)"
        :primaryAction="plansAction"
    />

    <div class="calendar-toolbar row items-center justify-between q-mb-md">
      <div>
        <div class="text-subtitle1 text-weight-medium">
          训练了 <strong>{{ activeDayCount }}</strong> 天，共 <strong>{{ totalExerciseCount }}</strong> 个动作
        </div>
      </div>

      <div class="row items-center q-gutter-sm">
        <q-btn flat round icon="chevron_left" @click="goToPreviousMonth"/>
        <q-btn flat round icon="today" @click="goToToday" />
        <q-btn flat round icon="chevron_right" @click="goToNextMonth"/>
      </div>
    </div>

    <div class="legend row q-col-gutter-sm q-mb-md">
      <div class="col-6 col-sm-auto" v-for="item in legendItems" :key="item.label">
        <div class="legend-item row items-center no-wrap">
          <span class="legend-swatch" :class="item.className"/>
          <span class="text-caption">{{ item.label }}</span>
        </div>
      </div>
    </div>

    <section v-if="showContinuePanel" class="continue-panel q-mb-md">
      <div class="row items-center justify-between q-col-gutter-md">
        <div class="col">
          <div class="text-caption text-grey-7">最近训练</div>
          <div class="text-subtitle1 text-weight-medium">{{ lastActiveRoutine.routineName }}</div>
          <div class="text-caption text-grey-7 q-mt-xs">
            上次训练：{{ formatRecordDate(lastActiveRoutine.lastTrainedAt) }}
          </div>
        </div>
        <div class="col-auto">
          <q-btn
              unelevated
              color="primary"
              icon="play_arrow"
              label="继续上次训练"
              @click="continueLastTraining"
          />
        </div>
      </div>
    </section>

    <div
        ref="viewportRef"
        class="calendar-viewport"
        @pointerdown="handlePointerDown"
        @pointermove="handlePointerMove"
        @pointerup="handlePointerUp"
        @pointercancel="handlePointerUp"
        @pointerleave="handlePointerLeave"
    >
      <div class="calendar-track" :style="trackStyle">
        <section
            v-for="panelDate in monthPanels"
            :key="monthKey(panelDate)"
            class="calendar-panel"
            :class="`weeks-${getWeekCount(panelDate)}`"
        >
          <div class="weekdays">
            <div v-for="weekday in weekdays" :key="weekday" class="weekday-cell">
              {{ weekday }}
            </div>
          </div>

          <div class="days-grid">
            <button
                v-for="cell in buildMonthGrid(panelDate)"
                :key="cell.dateKey"
                type="button"
                class="day-cell"
                :class="dayCellClasses(cell)"
                @click="handleDayClick(cell)"
            >
              <span class="day-number">{{ cell.day }}</span>
              <span v-if="cell.inCurrentMonth && cell.exerciseCount > 0" class="day-count">
                {{ cell.exerciseCount }} 动
              </span>
            </button>
          </div>
        </section>
      </div>
    </div>

    <section class="day-detail-panel q-mt-md">
      <div class="row items-center justify-between q-mb-sm">
        <div>
          <div class="text-caption text-grey-7">当天训练</div>
          <div class="text-subtitle1 text-weight-medium">{{ selectedDayTitle }}</div>
        </div>
        <div v-if="selectedDayDetails.length" class="text-caption text-grey-7">
          {{ selectedDayDetails.length }} 个动作
        </div>
      </div>

      <div v-if="dayDetailsLoading" class="detail-placeholder">
        正在加载当天记录...
      </div>
      <div v-else-if="!selectedDateKey" class="detail-placeholder">
        点击上方日期查看当天训练内容
      </div>
      <div v-else-if="selectedDayDetails.length === 0" class="detail-placeholder">
        这一天没有训练记录
      </div>
      <div v-else class="exercise-detail-list">
        <article
            v-for="item in selectedDayDetails"
            :key="item.exercise.id"
            class="exercise-detail-item"
            :class="{'is-expanded': expandedExerciseId === item.exercise.id}"
        >
          <button
              type="button"
              class="exercise-summary"
              @click="toggleExpandedExercise(item.exercise.id)"
          >
            <div class="exercise-summary-main">
              <div class="text-subtitle2 text-weight-medium">{{ item.exercise.name }}</div>
              <div class="text-caption text-grey-7">
                {{ item.routineName }} · {{ item.records.length }} 组 · {{ formatUnit(item.exercise.unit) }}
              </div>
            </div>
            <q-icon
                :name="expandedExerciseId === item.exercise.id ? 'expand_less' : 'expand_more'"
                size="20px"
                color="grey-7"
            />
          </button>

          <transition name="record-expand">
            <div v-if="expandedExerciseId === item.exercise.id" class="exercise-records">
              <div
                  v-for="record in item.records"
                  :key="record.id"
                  class="exercise-record-row"
              >
                <div class="text-body2 text-weight-medium">
                  {{ record.weight }} {{ formatUnit(item.exercise.unit) }}
                  <span v-if="record.reps" class="text-grey-7"> · {{ record.reps }} 次</span>
                </div>
                <div class="text-caption text-grey-7">
                  {{ formatRecordDate(record.createdAt) }}
                </div>
              </div>
            </div>
          </transition>
        </article>
      </div>
    </section>
  </q-page>
</template>

<script setup lang="ts">
import {computed, onMounted, reactive, ref} from 'vue';
import {useRouter} from "vue-router";
import {date, useQuasar} from "quasar";
import Header from "../components/Header.vue";
import api from "../utils/api.ts";
import {DailyExerciseCount, DayExerciseRecords, LastActiveRoutine} from "../bindings.ts";
import {HeaderPrimaryAction} from "../types.ts";
import {formatRecordDate} from "../utils/format.ts";
import {formatUnit} from "../utils/unitConvert.ts";

type CalendarCell = {
  date: Date;
  dateKey: string;
  day: number;
  inCurrentMonth: boolean;
  exerciseCount: number;
  level: number;
  isToday: boolean;
  isSelected: boolean;
};

type MonthStats = {
  counts: Record<number, number>;
  activeDayCount: number;
  totalExerciseCount: number;
};

const router = useRouter();
const $q = useQuasar();

const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
const legendItems = [
  {label: '未训练', className: 'level-0'},
  {label: '1-3 个动作', className: 'level-1'},
  {label: '4-5 个动作', className: 'level-2'},
  {label: '6+ 个动作', className: 'level-3'},
];

const currentMonth = ref(startOfMonth(new Date()));
const selectedDateKey = ref(formatDateKey(new Date()));
const selectedDayDetails = ref<DayExerciseRecords[]>([]);
const dayDetailsLoading = ref(false);
const expandedExerciseId = ref<number | null>(null);
const lastActiveRoutine = ref<LastActiveRoutine | null>(null);
const monthCache = reactive<Record<string, MonthStats | undefined>>({});
const loadingMonths = reactive<Record<string, boolean>>({});

const viewportRef = ref<HTMLElement | null>(null);
const dragState = reactive({
  active: false,
  pointerId: -1,
  startX: 0,
  offsetX: 0,
  animating: false,
});

const plansAction = computed<HeaderPrimaryAction>(() => ({
  icon: 'list_alt',
  label: '训练计划',
  action: () => router.push({name: 'Routines'})
}));

const monthPanels = computed(() => ([
  addMonths(currentMonth.value, -1),
  currentMonth.value,
  addMonths(currentMonth.value, 1),
]));

const currentMonthStats = computed(() => monthCache[monthKey(currentMonth.value)] || emptyMonthStats());
const activeDayCount = computed(() => currentMonthStats.value.activeDayCount);
const totalExerciseCount = computed(() => currentMonthStats.value.totalExerciseCount);
const showContinuePanel = computed(() => {
  if (!lastActiveRoutine.value) return false;
  const yesterday = date.subtractFromDate(new Date(), {days: 1});
  return !date.isSameDate(new Date(lastActiveRoutine.value.lastTrainedAt), yesterday, 'day');
});
const selectedDayTitle = computed(() => {
  if (!selectedDateKey.value) return '未选择日期';
  const parsed = new Date(`${selectedDateKey.value}T00:00:00`);
  return date.formatDate(parsed, 'YYYY年M月D日');
});

const trackStyle = computed(() => {
  const transition = dragState.animating ? 'transform 240ms ease' : 'none';
  return {
    transform: `translateX(calc(-33.333333% + ${dragState.offsetX}px))`,
    transition,
  };
});

function emptyMonthStats(): MonthStats {
  return {
    counts: {},
    activeDayCount: 0,
    totalExerciseCount: 0,
  };
}

function startOfMonth(source: Date) {
  return new Date(source.getFullYear(), source.getMonth(), 1);
}

function addMonths(source: Date, offset: number) {
  return new Date(source.getFullYear(), source.getMonth() + offset, 1);
}

function monthKey(source: Date) {
  return `${source.getFullYear()}-${String(source.getMonth() + 1).padStart(2, '0')}`;
}

function monthTitle(source: Date) {
  return `${source.getFullYear()}年${source.getMonth() + 1}月`;
}

function formatDateKey(source: Date) {
  return `${monthKey(source)}-${String(source.getDate()).padStart(2, '0')}`;
}

function getMonthDays(source: Date) {
  return new Date(source.getFullYear(), source.getMonth() + 1, 0).getDate();
}

function getWeekCount(source: Date) {
  const firstDay = new Date(source.getFullYear(), source.getMonth(), 1).getDay();
  return Math.ceil((firstDay + getMonthDays(source)) / 7);
}

function getLevel(count: number) {
  if (count >= 6) return 3;
  if (count >= 4) return 2;
  if (count >= 1) return 1;
  return 0;
}

function buildMonthGrid(source: Date): CalendarCell[] {
  const firstDay = new Date(source.getFullYear(), source.getMonth(), 1).getDay();
  const currentMonthDays = getMonthDays(source);
  const totalCells = Math.ceil((firstDay + currentMonthDays) / 7) * 7;
  const previousMonth = addMonths(source, -1);
  const previousMonthDays = getMonthDays(previousMonth);
  const monthStats = monthCache[monthKey(source)] || emptyMonthStats();
  const cells: CalendarCell[] = [];

  for (let index = 0; index < totalCells; index += 1) {
    const dayOffset = index - firstDay + 1;
    const isCurrentMonth = dayOffset > 0 && dayOffset <= currentMonthDays;
    const cellDate = isCurrentMonth
        ? new Date(source.getFullYear(), source.getMonth(), dayOffset)
        : dayOffset <= 0
            ? new Date(previousMonth.getFullYear(), previousMonth.getMonth(), previousMonthDays + dayOffset)
            : new Date(source.getFullYear(), source.getMonth() + 1, dayOffset - currentMonthDays);

    const exerciseCount = isCurrentMonth ? (monthStats.counts[cellDate.getDate()] || 0) : 0;
    const dateKey = formatDateKey(cellDate);

    cells.push({
      date: cellDate,
      dateKey,
      day: cellDate.getDate(),
      inCurrentMonth: isCurrentMonth,
      exerciseCount,
      level: isCurrentMonth ? getLevel(exerciseCount) : 0,
      isToday: dateKey === formatDateKey(new Date()),
      isSelected: dateKey === selectedDateKey.value,
    });
  }

  return cells;
}

function dayCellClasses(cell: CalendarCell) {
  return {
    'is-outside': !cell.inCurrentMonth,
    'is-today': cell.isToday,
    'is-selected': cell.isSelected && cell.inCurrentMonth,
    [`level-${cell.level}`]: cell.inCurrentMonth,
  };
}

async function loadMonth(source: Date) {
  const key = monthKey(source);
  if (monthCache[key] || loadingMonths[key]) {
    return;
  }

  loadingMonths[key] = true;
  try {
    const rows = await api.getDailyExerciseCount(source.getFullYear(), source.getMonth() + 1);
    const counts = rows.reduce<Record<number, number>>((acc, item: DailyExerciseCount) => {
      acc[item.day] = item.count;
      return acc;
    }, {});
    const totalExerciseCount = rows.reduce((sum, item: DailyExerciseCount) => sum + item.count, 0);

    monthCache[key] = {
      counts,
      activeDayCount: rows.length,
      totalExerciseCount,
    };
  } catch (e) {
    monthCache[key] = emptyMonthStats();
    $q.notify({type: 'negative', message: `加载 ${monthTitle(source)} 失败: ${e}`});
  } finally {
    loadingMonths[key] = false;
  }
}

async function preloadWindow(anchor: Date) {
  await Promise.all([
    loadMonth(addMonths(anchor, -1)),
    loadMonth(anchor),
    loadMonth(addMonths(anchor, 1)),
  ]);
}

async function loadLastActiveRoutine() {
  try {
    lastActiveRoutine.value = await api.getLastActiveRoutine();
  } catch (e) {
    lastActiveRoutine.value = null;
    $q.notify({type: 'negative', message: `加载最近训练失败: ${e}`});
  }
}

async function loadDayTrainingDetails(dateKey: string) {
  dayDetailsLoading.value = true;
  expandedExerciseId.value = null;
  try {
    selectedDayDetails.value = await api.getDayTrainingDetails(dateKey);
  } catch (e) {
    selectedDayDetails.value = [];
    $q.notify({type: 'negative', message: `加载当天训练失败: ${e}`});
  } finally {
    dayDetailsLoading.value = false;
  }
}

function goToToday() {
  currentMonth.value = startOfMonth(new Date());
  selectedDateKey.value = formatDateKey(new Date());
  preloadWindow(currentMonth.value);
  loadDayTrainingDetails(selectedDateKey.value);
}

function continueLastTraining() {
  if (!lastActiveRoutine.value) return;
  router.push({
    name: 'RoutineDetail',
    params: {id: lastActiveRoutine.value.routineId},
    state: {
      name: lastActiveRoutine.value.routineName
    }
  });
}

function handleDayClick(cell: CalendarCell) {
  if (!cell.inCurrentMonth) return;
  selectedDateKey.value = cell.dateKey;
  loadDayTrainingDetails(cell.dateKey);
}

function toggleExpandedExercise(exerciseId: number) {
  expandedExerciseId.value = expandedExerciseId.value === exerciseId ? null : exerciseId;
}

function handlePointerDown(event: PointerEvent) {
  if (dragState.animating) return;
  dragState.active = true;
  dragState.pointerId = event.pointerId;
  dragState.startX = event.clientX;
  dragState.offsetX = 0;
  viewportRef.value?.setPointerCapture(event.pointerId);
}

function handlePointerMove(event: PointerEvent) {
  if (!dragState.active || event.pointerId !== dragState.pointerId) return;
  dragState.offsetX = event.clientX - dragState.startX;
}

function handlePointerLeave(event: PointerEvent) {
  if (!dragState.active) return;
  if (event.pointerType === 'mouse') {
    handlePointerUp(event);
  }
}

function handlePointerUp(event: PointerEvent) {
  if (!dragState.active || event.pointerId !== dragState.pointerId) return;

  const width = viewportRef.value?.clientWidth || 1;
  const threshold = Math.min(120, width * 0.22);

  if (Math.abs(dragState.offsetX) > threshold) {
    animateToMonth(dragState.offsetX < 0 ? 1 : -1);
  } else {
    snapBack();
  }

  dragState.active = false;
  dragState.pointerId = -1;
}

function snapBack() {
  dragState.animating = true;
  dragState.offsetX = 0;
  window.setTimeout(() => {
    dragState.animating = false;
  }, 240);
}

function goToPreviousMonth() {
  animateToMonth(-1);
}

function goToNextMonth() {
  animateToMonth(1);
}

function animateToMonth(direction: -1 | 1) {
  if (dragState.animating) return;

  const width = viewportRef.value?.clientWidth || 0;
  dragState.animating = true;
  dragState.offsetX = direction === 1 ? -width : width;

  window.setTimeout(async () => {
    currentMonth.value = addMonths(currentMonth.value, direction);
    await preloadWindow(currentMonth.value);
    selectedDateKey.value = '';
    selectedDayDetails.value = [];
    expandedExerciseId.value = null;

    dragState.animating = false;
    dragState.offsetX = 0;
  }, 240);
}

onMounted(() => {
  preloadWindow(currentMonth.value);
  loadDayTrainingDetails(selectedDateKey.value);
  loadLastActiveRoutine();
});
</script>

<style scoped>
.calendar-page {
  background:
      radial-gradient(circle at top left, rgba(25, 118, 210, 0.12), transparent 28%),
      linear-gradient(180deg, #f8fbff 0%, #eef3f8 100%);
}

.calendar-toolbar {
  padding: 12px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(27, 42, 58, 0.08);
  box-shadow: 0 10px 24px rgba(27, 42, 58, 0.06);
}

.continue-panel {
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(27, 42, 58, 0.08);
  box-shadow: 0 10px 24px rgba(27, 42, 58, 0.06);
}

.legend-item {
  gap: 8px;
  padding: 6px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
}

.legend-swatch {
  width: 12px;
  height: 12px;
  border-radius: 999px;
  border: 1px solid rgba(27, 42, 58, 0.08);
}

.legend-swatch.level-0 {
  background: #edf1f5;
}

.legend-swatch.level-1 {
  background: #a9d6b8;
}

.legend-swatch.level-2 {
  background: #62b27a;
}

.legend-swatch.level-3 {
  background: #215c3a;
}

.calendar-viewport {
  overflow: hidden;
  touch-action: pan-y;
  flex: 0 0 auto;
  min-height: 0;
}

.day-detail-panel {
  padding: 14px 16px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(27, 42, 58, 0.08);
  box-shadow: 0 10px 24px rgba(27, 42, 58, 0.06);
}

.detail-placeholder {
  padding: 18px 0;
  color: #5f6f82;
  font-size: 13px;
}

.exercise-detail-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.exercise-detail-item {
  border: 1px solid rgba(27, 42, 58, 0.08);
  border-radius: 14px;
  background: rgba(248, 251, 255, 0.92);
  overflow: hidden;
  padding: 6px 12px;
}

.exercise-detail-item.is-expanded {
  background: rgba(255, 255, 255, 0.98);
}

.exercise-summary {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 4px;
  border: 0;
  background: transparent;
  text-align: left;
}

.exercise-summary-main {
  min-width: 0;
}

.exercise-records {
  border-top: 1px solid rgba(27, 42, 58, 0.08);
  padding: 4px 4px 10px;
}

.exercise-record-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
}

.exercise-record-row + .exercise-record-row {
  border-top: 1px dashed rgba(27, 42, 58, 0.08);
}

.record-expand-enter-active,
.record-expand-leave-active {
  transition: opacity 180ms ease, transform 180ms ease;
  transform-origin: top;
}

.record-expand-enter-from,
.record-expand-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.calendar-track {
  display: flex;
  width: 300%;
  height: 100%;
}

.calendar-panel {
  width: calc(100% / 3);
  flex: 0 0 calc(100% / 3);
  min-width: 0;
}

.weekdays,
.days-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 8px;
}

.weekdays {
  margin-bottom: 10px;
}

.weekday-cell {
  text-align: center;
  color: #5f6f82;
  font-size: 12px;
  font-weight: 600;
}

.days-grid {
  grid-auto-rows: 58px;
}

.calendar-panel.weeks-5 .days-grid {
  grid-auto-rows: 64px;
}

.day-cell {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: flex-start;
  padding: 10px;
  border: 1px solid rgba(27, 42, 58, 0.08);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.9);
  color: #1b2a3a;
  text-align: left;
}

.day-cell.is-outside {
  background: rgba(255, 255, 255, 0.22);
  border-color: rgba(27, 42, 58, 0.04);
  color: rgba(95, 111, 130, 0.42);
}

.day-cell.level-0:not(.is-outside) {
  background: rgba(237, 241, 245, 0.96);
}

.day-cell.level-1 {
  background: rgba(169, 214, 184, 0.95);
}

.day-cell.level-2 {
  background: rgba(98, 178, 122, 0.95);
  color: #16311f;
}

.day-cell.level-3 {
  background: rgba(33, 92, 58, 0.96);
  color: #f4fbf6;
}

.day-cell.is-selected {
  box-shadow: inset 0 0 0 2px #f59f00;
}

.day-cell.is-today .day-number {
  text-decoration: underline;
  text-underline-offset: 4px;
}

.day-number {
  font-size: 15px;
  font-weight: 700;
}

.day-count {
  font-size: 12px;
  opacity: 0.8;
}

@media (max-width: 600px) {
  .calendar-toolbar {
    padding: 12px;
  }

  .continue-panel {
    padding: 12px;
  }

  .days-grid {
    gap: 6px;
    grid-auto-rows: 50px;
  }

  .calendar-panel.weeks-5 .days-grid {
    grid-auto-rows: 56px;
  }

  .day-cell {
    padding: 8px;
  }

  .day-detail-panel {
    padding: 12px;
  }

  .exercise-detail-item {
    padding-left: 10px;
    padding-right: 10px;
  }
}
</style>
