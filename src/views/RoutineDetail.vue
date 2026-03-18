<template>
  <q-page class="q-pa-md column">
    <Header :title="routineName || '训练动作列表'" :showBack="true" />

    <div v-if="exercises.length === 0" class="col flex flex-center column text-grey">
      <q-icon name="fitness_center" size="64px"/>
      <div class="q-mt-md">还没有动作，点击右下角添加</div>
    </div>

    <div class="row q-col-gutter-md" v-else>
      <div class="col-12" v-for="ex in exercises" :key="ex.id">
        <ExerciseCard
            :exercise="ex"
            @click="goToExerciseHistory(ex)"
            @delete="handleDeleteExercise"
            @record="openRecordDialog"
            @edit="handleEditExercise"
        />
      </div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>
  </q-page>

  <q-dialog v-model="showAddDialog" @hide="resetForm">
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">{{ isEditing ? '编辑动作' : '添加新动作' }}</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <q-form @submit="handleSave" class="q-gutter-md">
          <q-input
              filled
              v-model="formState.name"
              label="动作名称"
              :rules="[val => !!val || '必填']"
              autofocus
          />

          <div class="row q-col-gutter-sm">
            <div class="col-6">
              <q-input filled type="number" v-model.number="formState.sets" label="目标组数"/>
            </div>
            <div class="col-6">
              <q-input filled v-model="formState.reps" label="目标次数"/>
            </div>
          </div>

          <q-select
              filled
              v-model="formState.unit"
              :options="unitOptions"
              label="重量单位"
              emit-value
              map-options
          />

          <q-input filled v-model="formState.note" label="备注 (可选)" type="textarea" rows="2"/>

          <div class="row justify-end q-gutter-sm q-mt-md">
            <q-btn label="取消" flat color="primary" v-close-popup/>
            <q-btn :label="isEditing ? '保存' : '添加'" type="submit" color="primary" :loading="submitting"/>
          </div>
        </q-form>
      </q-card-section>
    </q-card>
  </q-dialog>

  <q-dialog v-model="showRecordDialog">
    <q-card style="min-width: 300px">
      <q-card-section>
        <div class="text-h6">{{ recordingExercise?.name }}</div>
        <div class="text-caption text-grey">
          {{ recordingExercise?.note || '记录数据，一天允许记录多次，动作详图表会按天取平均' }}
        </div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <q-input
            filled
            type="number"
            v-model.number="recordForm.weight"
            label="重量"
            :suffix="formatUnit(recordingExercise?.unit || '')"
            autofocus
        />
        <q-input
            class="q-mt-sm"
            filled
            type="number"
            v-model.number="recordForm.reps"
            label="实际完成次数 (可选)"
            placeholder="默认留空"
            @keyup.enter="handleSaveRecord"
        />
        <!-- 候选按钮，仅在 commonReps 有值时显示 -->
        <div v-if="commonReps.length" class="row q-mt-xs q-gutter-xs">
          <q-btn
              v-for="rep in commonReps"
              :key="rep"
              dense
              flat
              color="primary"
              :label="rep + '次'"
              size="sm"
              @click="quickInput(rep)"
          />
        </div>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat label="取消" color="primary" v-close-popup/>
        <q-btn flat label="确认记录" color="primary" @click="handleSaveRecord" :loading="submitting"/>
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import {computed, onMounted, reactive, ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useQuasar} from 'quasar';
import {invokeStrict} from '../utils/invokeStrict';
import type {Exercise} from '../types';
import ExerciseCard from '../components/ExerciseCard.vue';
import {formatUnit, unitOptions} from "../utils/unitConvert.ts";
import {useExerciseStore} from "../stores/exerciseStore.ts";
import Header from "../components/Header.vue";

const route = useRoute();
const router = useRouter();
const $q = useQuasar();
const exerciseStore = useExerciseStore();

// 路由参数 ID
const routineId = Number(route.params.id);
const routineName = history.state.name;

// 状态
const exercises = ref<Exercise[]>([]);
const loading = ref(false);
const submitting = ref(false);
const showAddDialog = ref(false);
const showRecordDialog = ref(false);
const editingId = ref<number | null>(null);
const isEditing = computed(() => editingId.value !== null);

// 表单数据
const formState = reactive({
  name: '',
  sets: 4,
  reps: '10-15',
  unit: 'kg',
  note: ''
});

// 记录表单数据
const recordingExercise = ref<Exercise | null>(null);
const recordForm = reactive({
  weight: null as number | null,
  reps: null as number | null
});

// 常用次数列表
const commonReps = ref<number[]>([]);

// 加载列表
async function loadData() {
  try {
    exercises.value = await invokeStrict('get_exercises', {routineId}, loading);
    // 批量获取统计
    const ids = exercises.value.map(e => e.id);
    if (ids.length) {
      await exerciseStore.fetchForExercises(ids);
    }
  } catch (e) {
    $q.notify({type: 'negative', message: String(e)});
  }
}

// 点击编辑按钮
function handleEditExercise(id: number) {
  const target = exercises.value.find(e => e.id === id);
  if (!target) return;

  editingId.value = id;
  // 回填数据
  formState.name = target.name;
  formState.sets = target.targetSets;
  formState.reps = target.targetReps;
  formState.unit = target.unit;
  formState.note = target.note || '';

  showAddDialog.value = true;
}

// 统一保存 (新增或更新)
async function handleSave() {
  if (!formState.name) return;

  try {
    if (isEditing.value) {
      await invokeStrict('update_exercise', {
        exerciseId: editingId.value!,
        name: formState.name,
        sets: formState.sets,
        reps: formState.reps,
        note: formState.note,
        unit: formState.unit
      }, submitting);

      // 更新本地列表
      const index = exercises.value.findIndex(e => e.id === editingId.value);
      if (index !== -1) {
        exercises.value[index] = {
          ...exercises.value[index],
          ...formState
        };
      }
      $q.notify({type: 'positive', message: '动作已更新'});

    } else {
      await invokeStrict('add_exercise', {
        routineId,
        ...formState
      }, submitting);

      $q.notify({type: 'positive', message: '动作添加成功'});
      await loadData();
    }

    showAddDialog.value = false;
  } catch (e) {
    $q.notify({type: 'negative', message: (isEditing.value ? '更新' : '添加') + '失败: ' + e});
  }
}

// 重置表单 (弹窗关闭触发)
function resetForm() {
  editingId.value = null;
  formState.name = '';
  formState.sets = 4;
  formState.reps = '10-15';
  formState.unit = 'kg';
  formState.note = '';
}

// 删除动作
function handleDeleteExercise(id: number) {
  $q.dialog({
    title: '确认删除',
    message: '删除动作会连同历史记录一起删除，确定吗？',
    cancel: true
  }).onOk(async () => {
    try {
      await invokeStrict('delete_exercise', {exerciseId: id});
      exercises.value = exercises.value.filter(e => e.id !== id);
      exerciseStore.removeStats(id);
      $q.notify('动作已删除');
    } catch (e) {
      $q.notify({type: 'negative', message: String(e)});
    }
  });
}

// 打开记录弹窗时获取常用次数
async function openRecordDialog(exercise: Exercise) {
  recordingExercise.value = exercise;
  recordForm.weight = null;
  recordForm.reps = null;
  showRecordDialog.value = true;

  try {
    commonReps.value = await invokeStrict('get_common_reps', {exerciseId: exercise.id});
  } catch (e) {
    console.warn('获取常用次数失败', e);
    commonReps.value = [];
  }
}

function quickInput(rep: number) {
  recordForm.reps = rep;
  if (recordForm.weight) {
    // 自动提交
    handleSaveRecord();
  }
}

// 保存记录
async function handleSaveRecord() {
  if (!recordingExercise.value || !recordForm.weight) {
    $q.notify({type: 'warning', message: '请输入重量'});
    return;
  }

  try {
    await invokeStrict('add_record', {
      exerciseId: recordingExercise.value.id,
      weight: Number(recordForm.weight),
      reps: recordForm.reps ? Number(recordForm.reps) : null
    }, submitting);

    $q.notify({type: 'positive', message: `记录成功: ${recordForm.weight} ${recordingExercise.value.unit}`});
    showRecordDialog.value = false;
    await exerciseStore.fetchForExercise(recordingExercise.value.id);
  } catch (e) {
    $q.notify({type: 'negative', message: '记录失败: ' + e});
  }
}

// 跳转详情
function goToExerciseHistory(e: Exercise) {
  router.push({
    name: 'ExerciseDetail',
    params: {id: e.id},
    state: {
      exerciseName: e.name,
      exerciseUnit: e.unit
    }
  });
}

onMounted(() => {
  if (!routineId) {
    $q.notify({type: 'negative', message: '无效的轮次ID'});
    router.back();
    return;
  }
  loadData();
});
</script>