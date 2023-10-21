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

const emit = defineEmits(["close"]);

watch(props, ({ visible }) => {
    if (visible) {
        getUser();
    } else {
        user.value = undefined;
    }
})

function getUser() {
    invoke<User>("get_user", { handle: props.userHandle })
        .then(u => {
            user.value = u;
        })
        .catch(err => console.error("Failed to get user", err));
}

function hideUser() {
    emit("close");
}
</script>

<template>
    <div v-if="props.visible" class="user arrow">
        <div class="userHeader">
            <div :style="{ fontStyle: 'italic' }">{{ props.userHandle }}</div>
            <div class="close" @click="hideUser()">X</div>
        </div>

        <div v-if="user">
            <span id="myPopup" v-html="user?.about" class="about" />
            <div class="karma">
                <div>
                    Karma: {{ user?.karma }}
                </div>
                <div>
                    Registered: {{ user.created }}
                </div>
            </div>
        </div>
        <div v-else>
            Loading...
        </div>
    </div>
</template>

<style scoped>
.user {
    background-color: rgb(113, 146, 149);
    color: white;
    padding: 5px;
    border-radius: 8px;
    box-shadow: 2px 1px 1px gray;
    margin-top: 5px;
    margin-bottom: 5px;
    position: relative;
    display: inline-block;
    min-width: 10rem;
}

.userHeader {
    display: flex;
    justify-content: space-between;
    margin-right: 5px;
}

.arrow:before {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    left: 20px;
    right: auto;
    top: auto;
    bottom: 20px;
    border: 0px solid;
    border-color: #666 transparent transparent transparent;
}

.arrow:after {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    bottom: -38px;
    right: auto;
    top: auto;
    left: 5rem;
    border: 20px solid;
    border-color: rgb(113, 146, 149) transparent transparent transparent;
}

.karma {
    font-size: smaller;
}

.about {
    margin-bottom: 1rem;
    overflow: auto;
}
</style>