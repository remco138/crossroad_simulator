(ns ui.drawing
  (:require [cljsjs.paperjs]))



(defn init! [] (def  path1 (js/paper.Path.));(js-obj
                                           ; "point" (array 75 75)
                                            ;"size" (array 75 75)
                                            ;"strokeColor" "black"))))



(defn on-frame []
  (.rotate path1 9))

