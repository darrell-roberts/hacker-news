<script setup lang="ts">
import { reactive } from "vue";
import { Item } from "../types/article";
import { invoke } from "@tauri-apps/api/tauri";
import UserModal from "./UserModal.vue";
import RichText from "./RichText.vue";

interface Props {
    comment: Item;
}

interface State {
    commentsOpen: boolean;
    comments: Item[];
    fetching: boolean;
    error?: string;
    userVisible: boolean;
    commentVisible: boolean;
}

const props = defineProps<Props>();
const state = reactive<State>({
    commentsOpen: false,
    comments: [],
    fetching: false,
    userVisible: false,
    commentVisible: true,
});

function toggleComments() {
    if (!state.commentsOpen) {
        getComments();
    } else {
        state.error = undefined;
    }
    state.commentsOpen = !state.commentsOpen;
}

function getComments() {
    state.fetching = true;
    state.error = undefined;
    invoke<Item[]>("get_items", { items: props.comment.kids })
        .then((items) => (state.comments = items))
        .catch((err) => (state.error = err))
        .finally(() => (state.fetching = false));
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}

function toggleUserView() {
    state.userVisible = !state.userVisible;
}

function hideComment() {
    state.commentVisible = false;
}
</script>

<template>
    <div
        :class="{
            talkBubble: true,
            triRight: true,
            hideComment: !state.commentVisible,
        }"
    >
        <div style="display: flex; justify-content: space-between">
            <div></div>
            <div class="close" @click="hideComment()">X</div>
        </div>

        <RichText :richText="comment.text" />

        <UserModal
            :visible="state.userVisible"
            :user-handle="props.comment.by"
            @close="toggleUserView()"
        />

        <div class="bottom">
            <div class="author">
                by
                <span @click="toggleUserView()" class="by">
                    {{ props.comment.by }}
                </span>
                {{ props.comment.time }}
            </div>
            <div @click="toggleComments" class="commentFooter">
                <div
                    v-if="props.comment.kids.length > 0"
                    style="display: flex; flex-direction: row"
                >
                    <div>
                        <span
                            >{{ toggleText() }}
                            {{ props.comment.kids.length }}</span
                        >
                    </div>
                    <div style="margin-left: 5px">
                        <svg width="20" height="19">
                            <path
                                d="M7.725 19.872a.718.718 0 0 1-.607-.328.725.725 0 0 1-.118-.397V16H3.625A2.63 2.63 0 0 1 1 13.375v-9.75A2.629 2.629 0 0 1 3.625 1h12.75A2.63 2.63 0 0 1 19 3.625v9.75A2.63 2.63 0 0 1 16.375 16h-4.161l-4 3.681a.725.725 0 0 1-.489.191ZM3.625 2.25A1.377 1.377 0 0 0 2.25 3.625v9.75a1.377 1.377 0 0 0 1.375 1.375h4a.625.625 0 0 1 .625.625v2.575l3.3-3.035a.628.628 0 0 1 .424-.165h4.4a1.377 1.377 0 0 0 1.375-1.375v-9.75a1.377 1.377 0 0 0-1.374-1.375H3.625Z"
                            ></path>
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <div v-if="state.fetching">Loading...</div>

    <div v-if="state.error" class="error">
        Failed to load comments: {{ state.error }}
    </div>
    <div
        v-if="state.commentsOpen"
        v-for="comment of state.comments"
        class="childComments"
    >
        <Comment :comment="comment" />
    </div>
</template>

<style scoped>
.comment {
    overflow: auto;
    max-width: 35rem;
    font-size: 1.1rem;
}

.talkBubble {
    margin-left: 40px;
    margin-bottom: 5px;
    margin-top: 5px;
    display: inline-block;
    position: relative;
    height: auto;
    background-color: #f9fdc1;
    border-radius: 8px;
    padding: 10px;
    box-shadow: 1px 1px 2px -1px;
    color: black;
}

.commentFooter:hover {
    color: rgb(122, 14, 14);
}

.triRight:before {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    left: -40px;
    right: auto;
    top: -1px;
    bottom: auto;
    border: 0px solid;
    border-color: #666 transparent transparent transparent;
}

.triRight:after {
    content: " ";
    position: absolute;
    width: 0;
    height: 0;
    left: -20px;
    right: auto;
    top: 0px;
    bottom: auto;
    border: 22px solid;
    border-color: #f9fdc1 transparent transparent transparent;
}

.pointer {
    top: 40px;
    position: relative;
}

.hideComment {
    visibility: hidden;
    display: none;
}

.childComments {
    background-color: #e1e1e1;
    margin-left: 20px;
}
</style>
