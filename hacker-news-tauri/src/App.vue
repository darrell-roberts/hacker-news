<script setup lang="ts">
import Article from "./components/Article.vue";
import { TopStories, Item } from "./types/article";
import { onMounted, onUnmounted, reactive } from "vue";
import Tooltip from "./components/Tooltip.vue";
import { UnlistenFn, listen } from '@tauri-apps/api/event'

interface State {
  topStories: TopStories,
  fetching: boolean,
  error?: string,
  filtered: boolean,
  url?: string
  unlisten?: UnlistenFn | void | string ,
}

const state = reactive<State>({
    topStories: { items: [] },
    fetching: false,
    filtered: false,
});

onMounted(() => {
    listen<TopStories>("top_stories", (topStories) => {
        mergeViewed(topStories.payload);
    }).catch(err => state.error = `Failed to listen to top stories ${err}`)
    .then(unlisten => state.unlisten = unlisten);
});

onUnmounted(() => {
    if (typeof state.unlisten) {
        (state.unlisten as UnlistenFn)();
    }
});

function toggleFilter() {
  state.filtered = !state.filtered;
}

function applyFilter(item: Item) {
  if (state.filtered) {
    return item.hasRust;
  } else {
    return true;
  }
}

function mergeViewed(items: TopStories) {
    const viewed = state.topStories.items.filter(item => item.viewed).map(item => item.id);

    for (const item of items.items) {
        if (viewed.includes(item.id)) {
            item.viewed = true;
        }
    }
    state.topStories = items;
}
</script>

<template>
  <div class="container">
    <div v-if="state.error">Failed to load stories: {{ state.error }}</div>

    <div class="articles">
      <div v-for="(item, index) in state.topStories.items" :key="item.id">
        <Article
            :item="item"
            :index="index"
            v-if="applyFilter(item)"
            @viewed="() => item.viewed = true"
            @url="(url) => state.url = url"
        />
      </div>
    </div>

    <span class="url">{{ state.url }}</span>

    <div class="status-line">
        <Tooltip content="Reload">
        <div>
            <div v-if="state.fetching">Loading...</div>
            <div v-else class="status-action">
                {{ state.topStories.loaded }}
            </div>
        </div>
        </Tooltip>
        <Tooltip content="Filter Rust">
            <div @click="toggleFilter()" class="status-action">
                Rust articles: {{ state.topStories.rustArticles }}
            </div>
        </Tooltip>
    </div>
  </div>
</template>

<style scoped>
.articles {
  overflow: auto;
  height: calc(100vh - 65px);
}

.footer {
  margin-top: 10px;
  height: 50px;
}

.status-line {
  color: gray;
  font-size: small;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  margin-left: 5px;
  margin-right: 5px;
}

.status-action {
  cursor: pointer;

}

.status-action:hover {
  color: white;
}

.url {
    text-align: left;
    color: white;
    text-overflow: ellipsis;
    overflow: hidden;
    text-align: left;
    text-wrap: nowrap;
    height: 25px;
    width: 95vw;
    margin-left: 5px;
    white-space: nowrap;
}
</style>
