<script setup lang="ts">
import { reactive } from "vue";
import { Item } from "../types/article";
import UserModal from "./UserModal.vue";
import { shell } from "@tauri-apps/api";

interface Props {
    item: Item;
    index: number;
}

interface State {
    commentsOpen: boolean;
    userVisible: boolean;
}

const props = defineProps<Props>();
const state = reactive<State>({
    commentsOpen: false,
    userVisible: false,
});

const emit = defineEmits(["viewed", "url", "showComments"]);

function openLink() {
    emit("viewed");
    if (props.item.url) {
        shell.open(props.item.url);
    }
}

function toggleComments() {
    emit("showComments", props.item);
}

function toggleText() {
    return state.commentsOpen ? "[-]" : "[+]";
}

function hasRust() {
    return props.item.hasRust;
}

function positionChanged() {
    if (props.item.new) {
        return "🆕";
    }
    if (props.item.positionChange.type === "Up") {
        return "🔺";
    } else if (props.item.positionChange.type === "Down") {
        return "🔻";
    } else {
        return "";
    }
}

function toggleUserView() {
    state.userVisible = !state.userVisible;
}

function typeBadge() {
    switch (props.item.ty) {
        case "job":
            return "(job)";
        case "poll":
            return "(poll)";
        case "pollopt":
            return "(pollopt)";
        default:
            return "";
    }
}
</script>

<template>
    <div
        :class="{
            article: true,
            viewed: props.item.viewed,
            nonStory: props.item.ty !== 'story',
        }"
    >
        <div class="title-container">
            <div class="title">
                <span>{{ props.index + 1 }}. </span>
                <span
                    @click="openLink"
                    v-if="props.item.url"
                    v-on:mouseover="() => emit('url', props.item.url)"
                    v-on:mouseout="() => emit('url', '')"
                >
                    {{ props.item.title }}
                </span>
                <span
                    v-else
                    @click="toggleComments"
                    v-on:mouseover="() => emit('url', 'Text article')"
                    v-on:mouseout="() => emit('url', '')"
                >
                    {{ props.item.title }}
                </span>
            </div>

            <div v-if="hasRust()" class="rustArticle">
                <img src="/rust-logo-blk.svg" class="rustBadge" />
            </div>

            <div style="display: flex">
                <div style="margin-left: 10px">{{ typeBadge() }}</div>
                <div v-if="positionChanged() !== ''" class="positionChange">
                    {{ positionChanged() }}
                </div>
            </div>
        </div>

        <UserModal
            :visible="state.userVisible"
            :user-handle="props.item.by"
            @close="toggleUserView()"
        />

        <div class="bottom">
            <div class="author">
                {{ props.item.score }} points, by
                <span class="by" @click="toggleUserView()">{{
                    props.item.by
                }}</span>
                {{ props.item.time }}
            </div>
            <div @click="toggleComments" class="commentFooter">
                <div
                    v-if="props.item.kids.length > 0"
                    style="display: flex; flex-direction: row"
                >
                    <div>
                        <span
                            >{{ toggleText() }}
                            {{ props.item.kids.length }}</span
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
</template>

<style scoped>
.article {
    text-align: start;
    padding: 10px 10px 5px 10px;
    margin: 1px 5px 1px 5px;
    border-radius: 8px;
    background-color: white;
    color: black;
    display: inline-block;
    width: 95%;
    box-shadow: 2px 1px 1px;
    /* height: 100%; */
    /* width: 30rem; */
}

.nonStory {
    background-color: rgb(187, 187, 125);
}

.nonStory.viewed {
    background-color: rgb(149, 149, 100);
}

.rustArticle {
    display: flex;
    flex-grow: 1;
    justify-content: end;
}

.title {
    font-size: larger;
    font-weight: 500;
    cursor: pointer;
}

/* .commentFooter {
    background-color: azure;
    border-radius: 8px;
} */

.commentFooter:hover {
    color: rgb(122, 14, 14);
}

.title-container {
    flex-direction: row;
    display: flex;
    justify-content: space-between;
}

.rustBadge {
    width: 32px;
    height: 32px;
}

.positionChange {
    width: 10px;
    height: 10px;
    margin-right: 10px;
    margin-left: 10px;
}

.viewed {
    background-color: #e1e1e1;
}

@media (prefers-color-scheme: dark) {
    .article {
        color: #9fbfdf;
        background-color: #060d13;
        box-shadow: none;
    }
}
</style>
