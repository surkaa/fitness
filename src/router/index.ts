import RoutineDetail from "../views/RoutineDetail.vue";
import ExerciseDetail from "../views/ExerciseDetail.vue";
import Home from "../views/Home.vue";
import {createRouter, createWebHashHistory, RouteRecordRaw} from "vue-router";

const routes = [
    {path: '/', component: Home},
    {path: '/routine/:id', component: RoutineDetail},
    {path: '/exercise/:id', component: ExerciseDetail},
] as RouteRecordRaw[];

const router = createRouter({
    routes,
    history: createWebHashHistory(),
});

router.beforeEach((to, from, next) => {
    console.log(`Navigating from ${from.fullPath} to ${to.fullPath}`);
    next();
})

export default router;