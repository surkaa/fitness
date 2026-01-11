import RoutineDetail from "../views/RoutineDetail.vue";
import ExerciseDetail from "../views/ExerciseDetail.vue";
import Home from "../views/Home.vue";
import {createRouter, createWebHashHistory, RouteRecordRaw} from "vue-router";

const routes = [
    {
        path: '/',
        name: 'Home',
        component: Home
    },
    {
        path: '/routine/:id',
        name: 'RoutineDetail',
        component: RoutineDetail
    },
    {
        path: '/exercise/:id',
        name: 'ExerciseDetail',
        component: ExerciseDetail
    },
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