<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import Comment from "./Comment.vue";
import { Item } from "../types/article";

const dialog = ref<HTMLDialogElement>();

interface State {
    fetching: boolean;
    error?: string;
    comments: Item[];
    visible: boolean;
    item?: Item;
}

const state = reactive<State>({
    fetching: false,
    comments: [],
    visible: false,
});

const emit = defineEmits(["viewed"]);

const showItem = (item: Item) => {
    state.item = item;
    getComments()
        .catch((err) => (state.error = err))
        .finally(() => (state.fetching = false));
    dialog.value?.showModal();
    state.visible = true;
};

const close = () => {
    dialog.value?.close();
    state.item = undefined;
    state.comments = [];
    state.error = undefined;
    state.visible = false;
};

defineExpose({
    show: showItem,
    showItem,
    close,
});

async function getComments() {
    state.fetching = true;
    state.comments = await invoke<Item[]>("get_items", {
        items: state.item?.kids,
    });
}

watch(state, (s) => {
    if (s.visible) {
        dialog.value?.scrollTo(0, 0);
    }
});
</script>

<template>
    <dialog ref="dialog">
        <div class="top">
            <div class="title">{{ state.item?.title }}</div>
            <div
                @click="close"
                style="font-size: 2rem; margin-right: 10px"
                class="close"
            >
                X
            </div>
        </div>

        <div class="content">
            <div v-if="state.fetching">Loading...</div>

            <div v-if="state.error" class="error">
                Failed to load comments: {{ state.error }}
            </div>

            <div
                v-if="state.item?.text"
                class="text-talk-bubble text-tri-right right-top"
            >
                <span v-html="state.item.text" />
            </div>

            <div v-for="comment of state.comments">
                <Comment :comment="comment" />
            </div>
        </div>
    </dialog>
</template>

<style>
:modal {
    background-color: #e1e1e1;
    box-shadow: 3px 3px 10px rgba(0 0 0 / 0.5);
    max-height: 100vh;
    max-width: 50rem;
    text-align: left;
    padding: 0;
    border: 0;
    border-radius: 8px;
    overscroll-behavior: none;
}

.content {
    margin-left: 10px;
    margin-right: 10px;
    padding: 0 5px 0 5px;
    overflow: auto;
}

.title {
    font-size: 1.5rem;
    padding: 5px;
}

.top {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    padding: 5px;
    position: sticky;
    top: 0;
    z-index: 1;
    background-color: #e1e1e1;
}

.text-talk-bubble {
    margin-right: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: #e4f7fb;
    border-radius: 8px;
    padding: 10px;
    box-shadow: -1px 1px 2px -1px;
    color: black;
    font-size: 1.1rem;
}

.text-tri-right.right-top:before {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    right: -40px;
    left: auto;
    top: -1px;
    bottom: auto;
    border: 0 solid;
    border-color: #666 transparent transparent transparent;
}

.text-tri-right.right-top:after {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    right: -20px;
    left: auto;
    top: 0;
    bottom: auto;
    border: 22px solid;
    border-color: #e4f7fb transparent transparent transparent;
}

@media (prefers-color-scheme: dark) {
    :modal {
        color: #9fbfdf;
        background-color: #060d13;
        box-shadow: none;
    }

    .top {
        background-color: #060d13;
        color: white;
    }

    .text-talk-bubble {
        color: black;
        background-color: darkgray;
    }

    .text-tri-right.right-top:after {
        border-color: darkgray transparent transparent transparent;
    }
}
</style>
