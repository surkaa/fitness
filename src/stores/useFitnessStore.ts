import {defineStore} from "pinia";
import {ref} from "vue";
import {Routine} from "../types.ts";

export const useFitnessStore = defineStore('fitness', () => {
    const routines = ref<Routine[]>([]);
})