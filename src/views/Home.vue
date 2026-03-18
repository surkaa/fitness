<template>
  <q-page class="q-pa-md column">
    <Header title="训练计划" :rightAction="rightAction"/>

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

  <q-dialog v-model="showAddDialog" @hide="resetForm">
    <q-card style="min-width: 350px">
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
import type {Routine} from '../types';
import {useRouter} from "vue-router";
import {useQuasar} from "quasar";
import {invokeStrict} from "../utils/invokeStrict.ts";
import {open, save} from '@tauri-apps/plugin-dialog';
import {readFile, writeFile} from '@tauri-apps/plugin-fs';
import Header from "../components/Header.vue";

const router = useRouter();
const $q = useQuasar();

const routines = ref<Routine[]>([]);
const loading = ref(false);
const showAddDialog = ref(false);
const submitting = ref(false);

// 记录当前正在编辑的 ID，如果为 null 则表示是新建模式
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

// 打开编辑弹窗
function handleEdit(id: number) {
  const target = routines.value.find(r => r.id === id);
  if (!target) return;

  // 回填数据
  editingId.value = id;
  formState.name = target.name;
  formState.description = target.description || '';

  showAddDialog.value = true;
}

// 统一保存入口 (创建/更新)
async function handleSave() {
  if (!formState.name) return;

  try {
    if (isEditing.value) {
      await invokeStrict('update_routine', {
        routineId: editingId.value!,
        name: formState.name,
        desc: formState.description || ''
      }, submitting);

      // 更新本地列表
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
      const newId = await invokeStrict('create_routine', {
        name: formState.name,
        desc: formState.description || ''
      }, submitting);

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
  }
}

// 重置表单
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
  // 二次确认
  $q.dialog({
    title: '确认删除',
    message: '确定要删除该计划吗？此操作不可撤销。',
    cancel: true
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

async function handleExport() {
  try {
    // 1. 调用系统原生的保存对话框（Android 下会唤起 SAF 文件选择器）
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filePath = await save({
      defaultPath: `fitness_backup_${timestamp}.db`,
      filters: [{name: 'Database', extensions: ['db']}]
    });

    if (!filePath) return; // 用户取消了保存

    // 2. 向 Rust 请求数据库文件的完整字节
    const dbBytes = await invokeStrict('get_db_bytes', {}, loading, $q);

    // 3. 使用前端的 FS 插件直接写入 (Tauri FS 插件已完美处理 Android 的写入权限和路径问题)
    await writeFile(filePath, new Uint8Array(dbBytes));

    $q.notify({
      type: 'positive',
      message: `导出成功！`,
      timeout: 3000
    });
  } catch (e) {
    $q.notify({type: 'negative', message: '导出失败: ' + e});
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
      message: '恢复将覆盖当前所有数据，且需要重启应用。确定继续？',
      cancel: true,
      persistent: true
    }).onOk(async () => {
      try {
        await invokeStrict('import_db_from_bytes', {bytes: Array.from(fileBytes)}, loading, $q);

        $q.notify({
          type: 'positive',
          message: '恢复成功，点击重启立即生效',
          timeout: 0,
          actions: [{
            label: '立即重启',
            color: 'white',
            handler: async () => await invokeStrict('restart_app')
          }]
        });
      } catch (e) {
        $q.notify({type: 'negative', message: '恢复失败: ' + e});
      }
    });
  } catch (e) {
    $q.notify({type: 'negative', message: '获取文件失败: ' + e});
  }
}


onMounted(() => {
  invokeStrict('get_routines', {}, loading).then(list => {
    routines.value = list;
  });
})
</script>