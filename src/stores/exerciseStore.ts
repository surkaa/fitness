import {defineStore} from 'pinia';
import {computed, ref} from 'vue';
import {invokeStrict} from '../utils/invokeStrict';
import type {ExerciseStats} from '../types';

export const useExerciseStore = defineStore('exercise', () => {
    const stats = ref<Record<string, ExerciseStats>>({});          // 键为 exerciseId 的字符串形式
    const loadingStats = ref<Record<string, boolean>>({});        // 每个动作的请求状态
    const pendingFetches = ref<Record<string, Promise<ExerciseStats | undefined>>>({}); // 用于防止重复请求

    /**
     * 获取单个动作的统计，并存入 store
     * 若同时多次调用同一个 exerciseId，只发起一次请求
     */
    async function fetchForExercise(exerciseId: number): Promise<ExerciseStats | undefined> {
        const idStr = String(exerciseId);
        if (idStr in pendingFetches.value) {
            return pendingFetches.value[idStr];
        }
        if (loadingStats.value[idStr]) {
            // 如果正在加载但没有记录 promise，则等待一个短暂的时间（通常不会发生）
            await new Promise(resolve => setTimeout(resolve, 100));
            return stats.value[idStr];
        }

        loadingStats.value[idStr] = true;
        const promise = (async () => {
            try {
                const stat = await invokeStrict('get_exercise_stats', {exerciseId});
                stats.value[idStr] = stat;
                return stat;
            } catch (e) {
                console.error(`获取动作 ${exerciseId} 统计失败`, e);
                return undefined;
            } finally {
                loadingStats.value[idStr] = false;
                delete pendingFetches.value[idStr];
            }
        })();

        pendingFetches.value[idStr] = promise;
        return promise;
    }

    /**
     * 批量获取多个动作的统计
     */
    async function fetchForExercises(exerciseIds: number[]) {
        await Promise.all(exerciseIds.map(id => fetchForExercise(id)));
    }

    /**
     * 手动更新统计（例如本地修改后直接更新）
     */
    function setStats(exerciseId: number, stat: ExerciseStats) {
        stats.value[String(exerciseId)] = stat;
    }

    /**
     * 删除统计（当动作被删除时调用）
     */
    function removeStats(exerciseId: number) {
        delete stats.value[String(exerciseId)];
        delete loadingStats.value[String(exerciseId)];
        delete pendingFetches.value[String(exerciseId)];
    }

    /**
     * 清空所有统计
     */
    function clearStats() {
        stats.value = {};
        loadingStats.value = {};
        pendingFetches.value = {};
    }

    return {
        stats: computed(() => stats.value),
        loadingStats,
        fetchForExercise,
        fetchForExercises,
        setStats,
        removeStats,
        clearStats,
    };
});
