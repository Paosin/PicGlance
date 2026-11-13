export function memoryTarget(hidden: boolean, imageInUse: boolean) {
  return hidden || !imageInUse ? 'low' : 'normal';
}
