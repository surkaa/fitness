<template>
  <q-page class="routines-page q-pa-md column">
    <Header title="训练计划" :showBack="true" :rightAction="rightAction"/>

    <div class="row q-col-gutter-md q-pb-xl" v-if="routines.length">
      <div class="col-12 col-sm-6" v-for="r in routines" :key="r.id">
        <RoutineCard
            :routine="r"
            @click="goToRoutine(r)"
            @delete="handleDelete(r.id)"
            @edit="handleEdit(r.id)"
        />
      </div>
    </div>

    <div v-else-if="!loading" class="empty-state col flex flex-center column text-grey">
      <q-icon name="fitness_center" size="56px"/>
      <div class="q-mt-md text-h6" style="opacity: 0.78">
        还没有训练计划
      </div>
      <div class="text-caption">点击右下角添加</div>
    </div>

    <q-page-sticky position="bottom-right" :offset="[18, 18]">
      <q-btn fab icon="add" color="primary" @click="showAddDialog = true"/>
    </q-page-sticky>
  </q-page>

  <q-dialog v-model="showAddDialog" @hide="resetForm">
    <q-card class="dialog-card" style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">{{ isEditing ? '编辑训练计划' : '新建训练计划' }}</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <q-form @submit="handleSave" class="q-gutter-md">
          <q-input
              filled
              v-model="formState.name"
              label="计划名称"
              :rules="[val => !!val || '名称不能为空']"
              autofocus
          />

          <q-input
              filled
              v-model="formState.description"
              label="描述"
              type="textarea"
              rows="3"
          />

          <div class="row justify-end q-gutter-sm q-mt-md">
            <q-btn label="取消" flat color="primary" v-close-popup/>
            <q-btn
                :label="isEditing ? '保存' : '创建'"
                type="submit"
                color="primary"
                :loading="submitting"
            />
          </div>
        </q-form>
      </q-card-section>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import {computed, onMounted, reactive, ref} from 'vue';
import RoutineCard from '../components/RoutineCard.vue';
import {useRouter} from "vue-router";
import {useQuasar} from "quasar";
import {open, save} from '@tauri-apps/plugin-dialog';
import {readFile, writeFile} from '@tauri-apps/plugin-fs';
import Header from "../components/Header.vue";
import {Routine} from "../bindings.ts";
import api from "../utils/api.ts";

const router = useRouter();
const $q = useQuasar();

const routines = ref<Routine[]>([]);
const loading = ref(false);
const showAddDialog = ref(false);
const submitting = ref(false);
const editingId = ref<number | null>(null);
const isEditing = computed(() => editingId.value !== null);

const formState = reactive({
  name: '',
  description: ''
});

const rightAction = computed(() => [{
  icon: 'download', label: '数据导出', action: handleExport
}, {
  icon: 'upload', label: '数据导入', action: handleImport
}]);

function handleEdit(id: number) {
  const target = routines.value.find(r => r.id === id);
  if (!target) return;

  editingId.value = id;
  formState.name = target.name;
  formState.description = target.description || '';
  showAddDialog.value = true;
}

async function handleSave() {
  if (!formState.name) return;

  submitting.value = true;
  try {
    if (isEditing.value) {
      await api.updateRoutine(editingId.value!, formState.name, formState.description);

      const index = routines.value.findIndex(r => r.id === editingId.value);
      if (index !== -1) {
        routines.value[index] = {
          ...routines.value[index],
          name: formState.name,
          description: formState.description || null
        };
      }

      $q.notify({type: 'positive', message: '计划已更新'});
    } else {
      const newId = await api.createRoutine(formState.name, formState.description);

      routines.value.push({
        id: newId,
        name: formState.name,
        description: formState.description || null
      });

      $q.notify({type: 'positive', message: '计划创建成功'});
    }

    showAddDialog.value = false;
  } catch (e) {
    $q.notify({type: 'negative', message: (isEditing.value ? '更新' : '创建') + '失败: ' + e});
  } finally {
    submitting.value = false;
  }
}

function resetForm() {
  editingId.value = null;
  formState.name = '';
  formState.description = '';
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
  $q.dialog({
    title: '确认删除',
    message: '确定要删除该计划吗？此操作不可撤销。',
    cancel: true
  }).onOk(() => {
    api.deleteRoutine(id).then(() => {
      routines.value = routines.value.filter(r => r.id !== id);
      $q.notify({
        type: 'positive',
        message: '计划已删除'
      });
    });
  });
}

async function handleExport() {
  loading.value = true;
  try {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filePath = await save({
      defaultPath: `fitness_backup_${timestamp}.db`,
      filters: [{name: 'Database', extensions: ['db']}]
    });

    if (!filePath) return;

    const dbBytes = await api.getDbBytes();
    await writeFile(filePath, new Uint8Array(dbBytes));

    $q.notify({
      type: 'positive',
      message: '导出成功！',
      timeout: 3000
    });
  } catch (e) {
    $q.notify({type: 'negative', message: '导出失败: ' + e});
  } finally {
    loading.value = false;
  }
}

async function handleImport() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{name: 'Database', extensions: ['db']}]
    });

    if (!selected) return;
    const filePath = Array.isArray(selected) ? selected[0] : selected;
    const fileBytes = await readFile(filePath);

    $q.dialog({
      title: '确认恢复',
      message: '恢复将覆盖当前所有数据，且会自动重启应用。确定继续？',
      cancel: true,
      persistent: true
    }).onOk(async () => {
      loading.value = true;
      try {
        await api.importDbFromBytes(fileBytes);
        $q.notify({type: 'positive', message: '恢复成功，应用即将重启'});
        setTimeout(() => api.restartApp(), 1500);
      } catch (e) {
        $q.notify({type: 'negative', message: '恢复失败: ' + e});
      } finally {
        loading.value = false;
      }
    });
  } catch (e) {
    $q.notify({type: 'negative', message: '获取文件失败: ' + e});
  }
}

onMounted(() => {
  loading.value = true;
  api.getRoutines().then(list => {
    routines.value = list;
  }).finally(() => loading.value = false);
});
</script>

<style scoped>
.routines-page {
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
</style>
