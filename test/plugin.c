#ifdef ENABLE_NUMBER_OF_PLUGINS
int OfxGetNumberOfPlugins() {
  return 1;
}
#endif

#ifdef ENABLE_GET_PLUGIN
void *OfxGetPlugin(int) {
  return nullptr;
}
#endif
