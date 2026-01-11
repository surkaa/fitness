<template>
  <div class="q-pa-md">
    <div class="text-h4 q-mb-md">我的计划</div>

    <div class="row q-col-gutter-md">
      <div class="col-12 col-sm-6" v-for="r in routines" :key="r.id">
        <RoutineCard
            :routine="r"
            @click="goToRoutine(r.id)"
            @delete="handleDelete(r.id)"
        />
      </div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>
  </div>
</template>

<script setup lang="ts">
import {onMounted, ref} from 'vue';
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

function goToRoutine(id: number) {
  router.push({
    name: 'RoutineDetail',
    params: {id}
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