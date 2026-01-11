<template>
  <q-dialog v-model="showAddDialog" persistent>
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">新建训练计划</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <q-form @submit="handleCreateRoutine" class="q-gutter-md">
          <q-input
              filled
              v-model="newRoutine.name"
              label="计划名称"
              :rules="[val => !!val || '名称不能为空']"
              autofocus
          />

          <q-input
              filled
              v-model="newRoutine.description"
              label="描述"
              type="textarea"
              rows="3"
          />

          <div class="row justify-end q-gutter-sm q-mt-md">
            <q-btn label="取消" flat color="primary" v-close-popup/>
            <q-btn label="创建" type="submit" color="primary" :loading="submitting"/>
          </div>
        </q-form>
      </q-card-section>
    </q-card>
  </q-dialog>
  <q-page class="q-pa-md column">
    <div class="text-h4 q-mb-md">训练计划</div>
    <div class="row q-col-gutter-md" v-if="routines.length">
      <div class="col-12 col-sm-6" v-for="r in routines" :key="r.id">
        <RoutineCard
            :routine="r"
            @click="goToRoutine(r)"
            @delete="handleDelete(r.id)"
        />
      </div>
    </div>

    <div v-else-if="!loading" class="col flex flex-center column text-grey">
      <q-icon name="fitness_center" size="64px"/>
      <div class="q-mt-md text-h6" style="opacity: 0.7">
        还没有训练计划
      </div>
      <div class="text-caption">点击右下角添加</div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>
  </q-page>
</template>

<script setup lang="ts">
import {onMounted, reactive, ref} from 'vue';
import RoutineCard from '../components/RoutineCard.vue';
import type {Routine} from '../types';
import {useRouter} from "vue-router";
import {useQuasar} from "quasar";
import {invokeStrict} from "../utils/invokeStrict.ts";

const router = useRouter();
const $q = useQuasar();

const routines = ref<Routine[]>([]);
const loading = ref(false);
const showAddDialog = ref(false);
const newRoutine = reactive({
  name: '',
  description: ''
});
const submitting = ref(false);

// 创建逻辑
async function handleCreateRoutine() {
  if (!newRoutine.name) return;

  try {
    // 调用 Rust 接口
    // 注意：Rust 后端接收参数名为 name 和 desc
    const newId = await invokeStrict('create_routine', {
      name: newRoutine.name,
      desc: newRoutine.description || ''
    }, submitting);

    $q.notify({ type: 'positive', message: '计划创建成功' });
    showAddDialog.value = false;

    // 手动更新列表，避免重新请求后端
    routines.value.push({
      id: newId,
      name: newRoutine.name,
      description: newRoutine.description || null
    });

    // 重置表单
    newRoutine.name = '';
    newRoutine.description = '';

  } catch (e) {
    $q.notify({ type: 'negative', message: '创建失败: ' + e });
  }
}

function goToRoutine(r: Routine) {
  router.push({
    name: 'RoutineDetail',
    params: {id: r.id},
    state: {
      name: r.name
    }
  });
}

function handleDelete(id: number) {
  // 二次确认
  $q.dialog({
    title: '确认删除',
    message: '确定要删除该计划吗？此操作不可撤销。',
    cancel: true,
    persistent: true
  }).onOk(() => {
    invokeStrict('delete_routine', {
      routineId: id
    }).then(() => {
      routines.value = routines.value.filter(r => r.id !== id);
      $q.notify({
        type: 'positive',
        message: '计划已删除'
      });
    });
  })
}

onMounted(() => {
  invokeStrict('get_routines', {}, loading).then(list => {
    routines.value = list;
  });
})
</script>