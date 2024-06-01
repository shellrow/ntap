<script lang="ts" setup>
import Default from './layouts/Default.vue';
</script>

<template>
  <Default><router-view></router-view></Default>
</template>

<style>
html,
body {
  font-size: 10px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  padding: 0 !important;
  margin: 0;
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}
</style>
