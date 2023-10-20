<script setup lang="ts">
import { ref, watch } from 'vue';
import { User } from '../types/user';
import { invoke } from '@tauri-apps/api';

interface Props {
    visible: boolean;
    userHandle: string;
}

const props = defineProps<Props>();
const user = ref<User>();



watch(props, ({visible}) => {
    if (visible) {
        getUser();
    } else {
        user.value = undefined;
    }
})

function getUser() {
    invoke<User>("get_user", {handle: props.userHandle})
        .then(u => {
            user.value = u;
        })
        .catch(err => console.error("Failed to get user", err));
}

</script>

<template>
    <div v-if="props.visible" class="user">
        <div v-if="user">
            <span id="myPopup" v-html="user?.about"/>
            <div>
                Karma: {{ user?.karma }}
            </div>
        </div>
        <div v-else>
            Loading...
        </div>
    </div>
</template>

<style scoped>
.user {
    background-color: rgb(76, 130, 136);
    color: white;
    padding: 5px;
    border-radius: 8px;
    box-shadow:  2px 1px 1px gray;
    margin-top: 5px;
    margin-bottom: 5px;
}
</style>