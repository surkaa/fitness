<template>
  <q-dialog :model-value="modelValue" @update:model-value="$emit('update:modelValue', $event)" @hide="resetForm">
    <q-card style="min-width: 300px">
      <q-card-section>
        <div class="text-h6">{{ exerciseName }}</div>
        <div class="text-caption text-grey">
          {{ exerciseNote || '记录数据，一天允许记录多次，动作详图表会按天取平均' }}
        </div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <q-input
            filled
            type="number"
            v-model.number="form.weight"
            label="重量"
            :suffix="formatUnit(unit)"
            autofocus
        />
        <q-input
            class="q-mt-sm"
            filled
            type="number"
            v-model.number="form.reps"
            label="实际完成次数 (可选)"
            placeholder="默认留空"
            @keyup.enter="handleSubmit"
        />
        <!-- 常用次数候选按钮 -->
        <div v-if="commonReps.length" class="row q-mt-xs q-gutter-xs">
          <q-btn
              v-for="rep in commonReps"
              :key="rep"
              dense
              flat
              color="primary"
              class="common-rep-btn"
              :label="rep + '次'"
              size="sm"
              @click="quickInput(rep)"
          />
        </div>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat label="取消" color="primary" v-close-popup />
        <q-btn flat label="确认记录" color="primary" @click="handleSubmit" :loading="submitting" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useQuasar } from 'quasar'
import { formatUnit } from '../utils/unitConvert'
import { useExerciseStore } from '../stores/exerciseStore'
import api from '../utils/api'

interface Props {
  modelValue: boolean
  exerciseId: number
  exerciseName: string
  unit: string
  exerciseNote?: string | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'success'): void
}>()

const $q = useQuasar()
const exerciseStore = useExerciseStore()

const submitting = ref(false)
const commonReps = ref<number[]>([])

const form = reactive({
  weight: null as number | null,
  reps: null as number | null,
})

// 弹窗打开时重置表单并加载常用次数
watch(
    () => props.modelValue,
    async (val) => {
      if (val) {
        resetForm()
        await loadCommonReps()
      }
    }
)

async function loadCommonReps() {
  try {
    commonReps.value = await api.getCommonReps(props.exerciseId)
  } catch (e) {
    console.warn('获取常用次数失败', e)
    commonReps.value = []
  }
}

function resetForm() {
  form.weight = null
  form.reps = null
}

function quickInput(rep: number) {
  form.reps = rep
  if (form.weight !== null && form.weight !== undefined) {
    handleSubmit()
  }
}

async function handleSubmit() {
  if (form.weight === null || form.weight === undefined) {
    $q.notify({ type: 'warning', message: '请输入重量' })
    return
  }

  submitting.value = true
  try {
    await api.addRecord(props.exerciseId, form.weight, form.reps)
    $q.notify({ type: 'positive', message: `记录成功: ${form.weight} ${props.unit}` })
    emit('update:modelValue', false)
    // 刷新统计（store 会自动处理）
    await exerciseStore.fetchForExercise(props.exerciseId)
    // 通知父组件刷新历史列表等
    emit('success')
  } catch (e) {
    $q.notify({ type: 'negative', message: '记录失败: ' + e })
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.common-rep-btn {
  font-size: 13px;
}
</style>
