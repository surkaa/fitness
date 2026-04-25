import {createRouter, createWebHashHistory, RouteRecordRaw} from "vue-router";

const routes = [
    {
        path: '/',
        name: 'Home',
        component: () => import('../views/Home.vue')
    },
    {
        path: '/routines',
        name: 'Routines',
        component: () => import('../views/Routines.vue')
    },
    {
        path: '/routine/:id',
        name: 'RoutineDetail',
        component: () => import('../views/RoutineDetail.vue')
    },
    {
        path: '/exercise/:id',
        name: 'ExerciseDetail',
        component: () => import('../views/ExerciseDetail.vue')
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
