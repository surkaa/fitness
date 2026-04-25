<template>
  <q-page class="routine-detail-page q-pa-md column">
    <Header :title="routineName || '训练动作列表'" :showBack="true" />

    <div v-if="exercises.length === 0" class="empty-state col flex flex-center column text-grey">
      <q-icon name="fitness_center" size="56px"/>
      <div class="q-mt-md">还没有动作，点击右下角添加</div>
    </div>

    <div class="row q-col-gutter-md q-pb-xl" v-else>
      <div class="col-12" v-for="ex in exercises" :key="ex.id">
        <ExerciseCard
            :exercise="ex"
            :image-expanded="expandedImageExerciseId === ex.id"
            :image-loading="loadingImageExerciseId === ex.id"
            :image-url="exerciseImageUrls[ex.id] ?? null"
            @click="goToExerciseHistory(ex)"
            @delete="handleDeleteExercise"
            @record="openRecordDialog"
            @edit="handleEditExercise"
            @image="handleExerciseImageAction"
        />
      </div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>
  </q-page>

  <q-dialog v-model="showAddDialog" @hide="resetForm">
    <q-card class="dialog-card" style="min-width: 350px">
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

  <RecordDialog
      v-model="showRecordDialog"
      :exercise-id="recordingExercise?.id ?? 0"
      :exercise-name="recordingExercise?.name ?? ''"
      :unit="recordingExercise?.unit ?? 'kg'"
      :exercise-note="recordingExercise?.note"
      @success="loadData"
  />

  <input
      ref="exerciseImageInput"
      type="file"
      accept="image/*"
      capture="environment"
      class="hidden-image-input"
      @change="handleExerciseImageSelected"
  />
</template>

<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, reactive, ref} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import {useQuasar} from 'quasar';
import ExerciseCard from '../components/ExerciseCard.vue';
import {unitOptions} from "../utils/unitConvert.ts";
import {useExerciseStore} from "../stores/exerciseStore.ts";
import Header from "../components/Header.vue";
import {Exercise} from "../bindings.ts";
import api from "../utils/api.ts";
import RecordDialog from "../components/RecordDialog.vue";

const route = useRoute();
const router = useRouter();
const $q = useQuasar();
const exerciseStore = useExerciseStore();

// 路由参数 ID
const routineId = Number(route.params.id);
const routineName = ref<string>((history.state.name as string) || '');

// 状态
const exercises = ref<Exercise[]>([]);
const loading = ref(false);
const submitting = ref(false);
const showAddDialog = ref(false);
const showRecordDialog = ref(false);
const editingId = ref<number | null>(null);
const isEditing = computed(() => editingId.value !== null);
const exerciseImageInput = ref<HTMLInputElement | null>(null);
const pendingImageExercise = ref<Exercise | null>(null);
const expandedImageExerciseId = ref<number | null>(null);
const loadingImageExerciseId = ref<number | null>(null);
const exerciseImageUrls = reactive<Record<number, string>>({});

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
// 加载列表
async function loadData() {
  loading.value = true;
  try {
    exercises.value = await api.getExercises(routineId);
    if (
        expandedImageExerciseId.value !== null &&
        !exercises.value.some(exercise => exercise.id === expandedImageExerciseId.value && exercise.hasImage)
    ) {
      revokeExerciseImageUrl(expandedImageExerciseId.value);
      expandedImageExerciseId.value = null;
    }
    // 批量获取统计
    const ids = exercises.value.map(e => e.id);
    if (ids.length) {
      await exerciseStore.fetchForExercises(ids);
    }
  } catch (e) {
    $q.notify({type: 'negative', message: String(e)});
  } finally {
    loading.value = false;
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

  submitting.value = true;
  try {
    if (isEditing.value) {
      await api.updateExercise(editingId.value!, formState.name, formState.sets, formState.reps, formState.note, formState.unit);

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
      await api.addExercise(routineId, formState.name, formState.sets, formState.reps, formState.note, formState.unit);

      $q.notify({type: 'positive', message: '动作添加成功'});
      await loadData();
    }

    showAddDialog.value = false;
  } catch (e) {
    $q.notify({type: 'negative', message: (isEditing.value ? '更新' : '添加') + '失败: ' + e});
  } finally {
    submitting.value = false;
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
      await api.deleteExercise(id);
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
  recordingExercise.value = exercise
  showRecordDialog.value = true
}

function revokeExerciseImageUrl(exerciseId: number) {
  if (!exerciseImageUrls[exerciseId]) {
    return;
  }
  URL.revokeObjectURL(exerciseImageUrls[exerciseId]);
  delete exerciseImageUrls[exerciseId];
}

async function handleExerciseImageAction(exercise: Exercise) {
  if (!exercise.hasImage) {
    pendingImageExercise.value = exercise;
    exerciseImageInput.value?.click();
    return;
  }

  if (expandedImageExerciseId.value === exercise.id) {
    expandedImageExerciseId.value = null;
    return;
  }

  expandedImageExerciseId.value = exercise.id;
  if (exerciseImageUrls[exercise.id]) {
    return;
  }

  loadingImageExerciseId.value = exercise.id;
  try {
    const image = await api.getExerciseImage(exercise.id);
    if (!image) {
      throw new Error('未找到动作图片');
    }

    exerciseImageUrls[exercise.id] = URL.createObjectURL(
        new Blob([new Uint8Array(image.bytes)], {type: image.mimeType}),
    );
  } catch (e) {
    expandedImageExerciseId.value = null;
    $q.notify({type: 'negative', message: `加载图片失败: ${e}`});
  } finally {
    loadingImageExerciseId.value = null;
  }
}

async function handleExerciseImageSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  const exercise = pendingImageExercise.value;

  input.value = '';
  pendingImageExercise.value = null;

  if (!file || !exercise) {
    return;
  }

  loadingImageExerciseId.value = exercise.id;
  try {
    const buffer = await file.arrayBuffer();
    await api.saveExerciseImage(exercise.id, file.type || 'image/jpeg', Array.from(new Uint8Array(buffer)));

    revokeExerciseImageUrl(exercise.id);
    exerciseImageUrls[exercise.id] = URL.createObjectURL(file);
    expandedImageExerciseId.value = exercise.id;

    const target = exercises.value.find(item => item.id === exercise.id);
    if (target) {
      target.hasImage = true;
    }

    $q.notify({type: 'positive', message: '动作照片已保存'});
  } catch (e) {
    $q.notify({type: 'negative', message: `保存图片失败: ${e}`});
  } finally {
    loadingImageExerciseId.value = null;
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
  if (!routineName.value) {
    api.getRoutine(routineId)
        .then(routine => {
          if (routine) {
            routineName.value = routine.name;
          }
        })
        .catch(e => {
          $q.notify({type: 'negative', message: `获取计划名称失败: ${e}`});
        });
  }
  loadData();
});

onBeforeUnmount(() => {
  Object.keys(exerciseImageUrls).forEach(key => revokeExerciseImageUrl(Number(key)));
});
</script>

<style scoped>
.routine-detail-page {
  background:
      radial-gradient(circle at top left, rgba(25, 118, 210, 0.1), transparent 24%),
      linear-gradient(180deg, #f8fbff 0%, #eef3f8 100%);
}

.empty-state {
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.72);
  border: 1px dashed rgba(27, 42, 58, 0.12);
}

.dialog-card {
  border-radius: 18px;
}

.hidden-image-input {
  display: none;
}

@media (max-width: 600px) {
}
</style>
