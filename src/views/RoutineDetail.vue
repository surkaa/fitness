<template>
  <q-page class="q-pa-md column">
    <div class="row items-center q-mb-md">
      <q-btn flat round icon="arrow_back" color="primary" @click="router.back()"/>
      <div class="text-h6 q-ml-md">{{ routineName || '训练动作列表' }}</div>
    </div>

    <div v-if="exercises.length === 0" class="col flex flex-center column text-grey">
      <q-icon name="fitness_center" size="64px"/>
      <div class="q-mt-md">还没有动作，点击右下角添加</div>
    </div>

    <div class="row q-col-gutter-md" v-else>
      <div class="col-12" v-for="ex in exercises" :key="ex.id">
        <ExerciseCard
            :exercise="ex"
            @click="goToExerciseHistory(ex.id)"
            @delete="handleDeleteExercise"
            @record="openRecordDialog"
        />
      </div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>

    <q-dialog v-model="showAddDialog" persistent>
      <q-card>
        <q-card-section>
          <div class="text-h6">添加新动作</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-form @submit="handleAddExercise" class="q-gutter-md">
            <q-input
                filled
                v-model="newExercise.name"
                label="动作名称"
                :rules="[val => !!val || '必填']"
                autofocus
            />

            <div class="row q-col-gutter-sm">
              <div class="col-6">
                <q-input filled type="number" v-model.number="newExercise.sets" label="目标组数"/>
              </div>
              <div class="col-6">
                <q-input filled v-model="newExercise.reps" label="目标次数"/>
              </div>
            </div>

            <q-select
                filled
                v-model="newExercise.unit"
                :options="unitOptions"
                label="重量单位"
                emit-value
                map-options
            />

            <q-input filled v-model="newExercise.note" label="备注 (可选)" type="textarea" rows="2"/>

            <div class="q-mt-md">
              <q-btn label="取消" flat color="primary" v-close-popup/>
              <q-btn label="保存" type="submit" color="primary" :loading="submitting"/>
            </div>
          </q-form>
        </q-card-section>
      </q-card>
    </q-dialog>

    <q-dialog v-model="showRecordDialog">
      <q-card style="min-width: 300px">
        <q-card-section>
          <div class="text-h6">{{ recordingExercise?.name }}</div>
          <div class="text-caption text-grey">记录今日最大重量</div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
              filled
              type="number"
              v-model.number="recordForm.weight"
              label="重量"
              :suffix="formatUnit(recordingExercise?.unit || '')"
              autofocus
              @keyup.enter="handleSaveRecord"
          />
          <q-input
              class="q-mt-sm"
              filled
              type="number"
              v-model.number="recordForm.reps"
              label="实际完成次数 (可选)"
              placeholder="默认留空"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat label="取消" color="primary" v-close-popup/>
          <q-btn flat label="确认记录" color="primary" @click="handleSaveRecord" :loading="submitting"/>
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import {onMounted, reactive, ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useQuasar} from 'quasar';
import {invokeStrict} from '../utils/invokeStrict';
import type {Exercise} from '../types';
import ExerciseCard from '../components/ExerciseCard.vue';
import {formatUnit, unitOptions} from "../utils/unitConvert.ts";

const route = useRoute();
const router = useRouter();
const $q = useQuasar();

// 路由参数 ID
const routineId = Number(route.params.id);
const routineName = history.state.name;

// 状态 TODO 以后还可以统计exercises训练了多少次
const exercises = ref<Exercise[]>([]);
const loading = ref(false);
const submitting = ref(false);
const showAddDialog = ref(false);
const showRecordDialog = ref(false);

// 添加表单数据
const newExercise = reactive({
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

// 加载列表
async function loadData() {
  try {
    exercises.value = await invokeStrict('get_exercises', {routineId}, loading);
  } catch (e) {
    $q.notify({type: 'negative', message: String(e)});
  }
}

// 添加动作
async function handleAddExercise() {
  if (!newExercise.name) return;

  try {
    await invokeStrict('add_exercise', {
      routineId,
      ...newExercise
    }, submitting);

    $q.notify({type: 'positive', message: '动作添加成功'});
    showAddDialog.value = false;

    // 重置表单
    newExercise.name = '';
    newExercise.note = '';

    // 刷新列表
    await loadData();
  } catch (e) {
    $q.notify({type: 'negative', message: '添加失败: ' + e});
  }
}

// 删除动作
function handleDeleteExercise(id: number) {
  $q.dialog({
    title: '确认删除',
    message: '删除动作会连同历史记录一起删除，确定吗？',
    cancel: true,
    persistent: true
  }).onOk(async () => {
    try {
      await invokeStrict('delete_exercise', {exerciseId: id});
      exercises.value = exercises.value.filter(e => e.id !== id);
      $q.notify('动作已删除');
    } catch (e) {
      $q.notify({type: 'negative', message: String(e)});
    }
  });
}

// 打开记录弹窗
function openRecordDialog(exercise: Exercise) {
  recordingExercise.value = exercise;
  recordForm.weight = null;
  recordForm.reps = null;
  showRecordDialog.value = true;
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
  } catch (e) {
    $q.notify({type: 'negative', message: '记录失败: ' + e});
  }
}

// 跳转详情
function goToExerciseHistory(id: number) {
  router.push({name: 'ExerciseDetail', params: {id}});
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